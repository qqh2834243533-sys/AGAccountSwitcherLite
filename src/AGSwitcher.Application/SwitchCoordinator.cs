using System.Globalization;
using System.Security.Cryptography;
using AGSwitcher.Domain;

namespace AGSwitcher.Application;

public sealed class SwitchCoordinator(
    AccountCatalog accounts,
    ISwitchClient client,
    ISwitchAudit audit)
{
    private readonly object _gate = new();
    private SwitchPhase _phase = SwitchPhase.Locked;
    private Guid _targetId;
    private string? _challenge;
    private bool _requiresRecovery;

    public (SwitchPhase Phase, string? Challenge) Status
    {
        get
        {
            lock (_gate)
            {
                return (_phase, _challenge);
            }
        }
    }

    public SwitchResult Begin(Guid targetId)
    {
        lock (_gate)
        {
            if (_phase != SwitchPhase.Locked)
            {
                return new(SwitchPhase.Blocked, SwitchIssue.ConcurrentOperation);
            }

            if (_requiresRecovery)
            {
                return new(SwitchPhase.Blocked, SwitchIssue.RecoveryRequired);
            }

            if (!accounts.Contains(targetId))
            {
                return new(SwitchPhase.Blocked, SwitchIssue.UnknownTarget);
            }

            _targetId = targetId;
            _challenge = RandomNumberGenerator.GetInt32(1000, 10000).ToString(CultureInfo.InvariantCulture);
            return Transition(SwitchPhase.Confirming);
        }
    }

    public SwitchResult Cancel()
    {
        lock (_gate)
        {
            return _phase == SwitchPhase.Confirming
                ? Finish(SwitchPhase.Locked, SwitchIssue.Cancelled)
                : new(SwitchPhase.Blocked, SwitchIssue.NoConfirmation);
        }
    }

    public async Task<SwitchResult> ConfirmAsync(string input)
    {
        Guid target;
        lock (_gate)
        {
            if (_phase != SwitchPhase.Confirming)
            {
                return new(SwitchPhase.Blocked, _phase == SwitchPhase.Locked
                    ? SwitchIssue.NoConfirmation
                    : SwitchIssue.ConcurrentOperation);
            }

            if (!string.Equals(input, _challenge, StringComparison.Ordinal))
            {
                return new(SwitchPhase.Confirming, SwitchIssue.InvalidConfirmation);
            }

            _challenge = null;
            target = _targetId;
            Transition(SwitchPhase.Precheck);
        }

        bool switchingStarted = false;
        try
        {
            var snapshot = client.Inspect();
            var issue = CheckSnapshot(snapshot);
            if (!accounts.Contains(target))
            {
                issue = SwitchIssue.UnknownTarget;
            }
            else if (issue == SwitchIssue.None && snapshot.ActiveAccountId == target)
            {
                issue = SwitchIssue.AlreadyActive;
            }

            if (issue != SwitchIssue.None)
            {
                return Finish(SwitchPhase.Blocked, issue);
            }

            Transition(SwitchPhase.Switching);
            switchingStarted = true;
            var attempt = await client.SwitchAsync(target);
            if (attempt != SwitchAttempt.Applied)
            {
                var failure = attempt switch
                {
                    SwitchAttempt.RestartFailed => SwitchIssue.RestartFailed,
                    SwitchAttempt.AuthUnavailable => SwitchIssue.AuthUnavailable,
                    _ => SwitchIssue.SwitchFailed
                };
                return Finish(SwitchPhase.Failed, failure, requiresRecovery: true);
            }

            Transition(SwitchPhase.Verifying);
            var verified = client.Inspect();
            if (CheckSnapshot(verified) != SwitchIssue.None || verified.ActiveAccountId != target)
            {
                return Finish(SwitchPhase.Failed, SwitchIssue.VerificationFailed, requiresRecovery: true);
            }

            return Finish(SwitchPhase.Completed);
        }
        catch (Exception)
        {
            return Finish(SwitchPhase.Failed, SwitchIssue.UnexpectedFailure, switchingStarted);
        }
    }

    public SwitchResult Reconcile()
    {
        lock (_gate)
        {
            if (_phase != SwitchPhase.Locked)
            {
                return new(SwitchPhase.Blocked, SwitchIssue.ConcurrentOperation);
            }

            try
            {
                var issue = CheckSnapshot(client.Inspect());
                if (issue != SwitchIssue.None)
                {
                    return new(SwitchPhase.Blocked, issue);
                }

                _requiresRecovery = false;
                return new(SwitchPhase.Locked);
            }
            catch (Exception)
            {
                return new(SwitchPhase.Failed, SwitchIssue.UnexpectedFailure);
            }
        }
    }

    private SwitchIssue CheckSnapshot(ClientSnapshot snapshot)
    {
        if (snapshot.Installed == false)
        {
            return SwitchIssue.ClientMissing;
        }

        if (snapshot.Installed != true || snapshot.Running is null
            || snapshot.ManagedInstanceCount is null || snapshot.UnfinishedOperation is null)
        {
            return SwitchIssue.ClientStateUnknown;
        }

        return snapshot.Compatibility switch
        {
            Capability.Unsupported => SwitchIssue.IncompatibleVersion,
            not Capability.Supported => SwitchIssue.CompatibilityUnknown,
            _ when snapshot.Running == false => SwitchIssue.ClientNotRunning,
            _ when snapshot.ManagedInstanceCount != 1 => SwitchIssue.ManagedInstanceConflict,
            _ when snapshot.UnfinishedOperation == true => SwitchIssue.UnfinishedOperation,
            _ when snapshot.Agent == AgentActivity.Busy => SwitchIssue.AgentBusy,
            _ when snapshot.Agent != AgentActivity.Idle => SwitchIssue.AgentStateUnknown,
            _ when snapshot.ActiveAccountId is not Guid current || !accounts.Contains(current)
                => SwitchIssue.ActiveIdentityUnknown,
            _ => SwitchIssue.None
        };
    }

    private SwitchResult Finish(SwitchPhase phase, SwitchIssue issue = SwitchIssue.None, bool requiresRecovery = false)
    {
        lock (_gate)
        {
            _challenge = null;
            _targetId = default;
            _requiresRecovery = requiresRecovery;
            var result = Transition(phase, issue);
            if (phase != SwitchPhase.Locked)
            {
                Transition(SwitchPhase.Locked);
            }

            return result;
        }
    }

    private SwitchResult Transition(SwitchPhase phase, SwitchIssue issue = SwitchIssue.None)
    {
        lock (_gate)
        {
            _phase = phase;
            var result = new SwitchResult(phase, issue);
            audit.Write(result);
            return result;
        }
    }
}
