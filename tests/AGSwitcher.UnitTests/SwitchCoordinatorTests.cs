using AGSwitcher.Application;
using AGSwitcher.Domain;

namespace AGSwitcher.UnitTests;

public sealed class SwitchCoordinatorTests
{
    private readonly AccountCatalog _accounts = new();
    private readonly RecordingAudit _audit = new();

    [Fact]
    public async Task ChallengeUsesExactlyFourAsciiDigitsAndCancellationInvalidatesIt()
    {
        var (coordinator, client) = Create();
        for (int index = 0; index < 24; index++)
        {
            Assert.Equal(SwitchPhase.Confirming, coordinator.Begin(TargetId).Phase);
            Assert.Matches("^[1-9][0-9]{3}$", coordinator.Status.Challenge!);
            var challenge = coordinator.Status.Challenge!;
            Assert.Equal(SwitchIssue.Cancelled, coordinator.Cancel().Issue);
            Assert.Equal((SwitchPhase.Locked, null), coordinator.Status);
            Assert.Equal(SwitchIssue.NoConfirmation, (await coordinator.ConfirmAsync(challenge)).Issue);
        }
        Assert.Equal(0, client.SwitchCount);
    }

    [Fact]
    public async Task WrongInputNeverInvokesClientAndSuccessRestoresLock()
    {
        var (coordinator, client) = Create();
        coordinator.Begin(TargetId);
        var challenge = coordinator.Status.Challenge!;
        var rejected = await coordinator.ConfirmAsync(challenge + " ");
        Assert.Equal(new SwitchResult(SwitchPhase.Confirming, SwitchIssue.InvalidConfirmation), rejected);
        Assert.Equal(0, client.SwitchCount);
        Assert.Equal(0, client.InspectCount);

        Assert.Equal(new SwitchResult(SwitchPhase.Completed), await coordinator.ConfirmAsync(challenge));
        Assert.Equal((SwitchPhase.Locked, null), coordinator.Status);
        Assert.Equal(TargetId, client.Snapshot.ActiveAccountId);
        Assert.Equal(1, client.SwitchCount);
        Assert.Equal(2, client.InspectCount);
        Assert.Equal(
            [SwitchPhase.Confirming, SwitchPhase.Precheck, SwitchPhase.Switching, SwitchPhase.Verifying,
                SwitchPhase.Completed, SwitchPhase.Locked],
            _audit.Events.Select(result => result.Phase));
        Assert.Equal(SwitchIssue.NoConfirmation, (await coordinator.ConfirmAsync(challenge)).Issue);
    }

    [Fact]
    public async Task SecondTransactionCannotBeginWhileConfirmingOrSwitching()
    {
        var (coordinator, client) = Create();
        var wait = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        client.Block = wait.Task;
        coordinator.Begin(TargetId);
        Assert.Equal(SwitchIssue.ConcurrentOperation, coordinator.Begin(_accounts.Accounts[2].Id).Issue);

        var operation = coordinator.ConfirmAsync(coordinator.Status.Challenge!);
        try
        {
            Assert.Equal(SwitchPhase.Switching, coordinator.Status.Phase);
            Assert.Equal(SwitchIssue.ConcurrentOperation,
                await Task.Run(() => coordinator.Begin(_accounts.Accounts[2].Id).Issue));
            Assert.Equal(SwitchIssue.ConcurrentOperation, (await coordinator.ConfirmAsync("1234")).Issue);
            Assert.Equal(SwitchIssue.NoConfirmation, coordinator.Cancel().Issue);
            Assert.Equal(SwitchIssue.ConcurrentOperation, coordinator.Reconcile().Issue);
        }
        finally
        {
            wait.SetResult();
        }

        Assert.Equal(SwitchPhase.Completed, (await operation).Phase);
        Assert.Equal(1, client.SwitchCount);
    }

    [Fact]
    public async Task UnknownOrWrongVerifiedIdentityDoesNotCompleteAndRequiresRecovery()
    {
        var (coordinator, client) = Create();
        client.ReturnWrongIdentity = true;
        coordinator.Begin(TargetId);

        var result = await coordinator.ConfirmAsync(coordinator.Status.Challenge!);
        Assert.Equal(new SwitchResult(SwitchPhase.Failed, SwitchIssue.VerificationFailed), result);
        Assert.Equal(SwitchIssue.RecoveryRequired, coordinator.Begin(TargetId).Issue);
        Assert.NotEqual(TargetId, client.Snapshot.ActiveAccountId);
    }

    [Fact]
    public async Task AlreadyActiveAndUnregisteredAccountsAreRejected()
    {
        var (coordinator, client) = Create();
        Assert.Equal(SwitchIssue.UnknownTarget, coordinator.Begin(Guid.NewGuid()).Issue);
        coordinator.Begin(_accounts.Accounts[0].Id);
        Assert.Equal(SwitchIssue.AlreadyActive, (await coordinator.ConfirmAsync(coordinator.Status.Challenge!)).Issue);
        Assert.Equal(0, client.SwitchCount);
    }

    [Fact]
    public async Task UnknownClientObservationBlocksBeforeSwitch()
    {
        var (coordinator, client) = Create();
        client.Snapshot = client.Snapshot with { Running = null };
        coordinator.Begin(TargetId);
        Assert.Equal(SwitchIssue.ClientStateUnknown,
            (await coordinator.ConfirmAsync(coordinator.Status.Challenge!)).Issue);
        Assert.Equal(0, client.SwitchCount);
    }

    [Fact]
    public async Task AdapterExceptionAfterSwitchStartsRequiresReconciliation()
    {
        var (coordinator, client) = Create();
        client.ThrowOnSwitch = true;
        coordinator.Begin(TargetId);
        Assert.Equal(SwitchIssue.UnexpectedFailure,
            (await coordinator.ConfirmAsync(coordinator.Status.Challenge!)).Issue);
        Assert.Equal(SwitchIssue.RecoveryRequired, coordinator.Begin(TargetId).Issue);
        Assert.Equal(SwitchPhase.Locked, coordinator.Reconcile().Phase);
    }

    private Guid TargetId => _accounts.Accounts[1].Id;

    private (SwitchCoordinator Coordinator, ControlledClient Client) Create()
    {
        var client = new ControlledClient(_accounts.Accounts[0].Id);
        return (new SwitchCoordinator(_accounts, client, _audit), client);
    }

    private sealed class ControlledClient(Guid current) : ISwitchClient
    {
        public ClientSnapshot Snapshot { get; set; } = new(true, Capability.Supported, true, 1, false, AgentActivity.Idle, current);
        public Task Block { get; set; } = Task.CompletedTask;
        public bool ReturnWrongIdentity { get; set; }
        public bool ThrowOnSwitch { get; set; }
        public int InspectCount { get; private set; }
        public int SwitchCount { get; private set; }

        public ClientSnapshot Inspect()
        {
            InspectCount++;
            return Snapshot;
        }

        public async Task<SwitchAttempt> SwitchAsync(Guid targetId)
        {
            SwitchCount++;
            await Block;
            if (ThrowOnSwitch)
            {
                throw new InvalidOperationException();
            }

            Snapshot = Snapshot with { ActiveAccountId = ReturnWrongIdentity ? current : targetId };
            return SwitchAttempt.Applied;
        }
    }
}

internal sealed class RecordingAudit : ISwitchAudit
{
    public List<SwitchResult> Events { get; } = [];
    public void Write(SwitchResult result) => Events.Add(result);
}
