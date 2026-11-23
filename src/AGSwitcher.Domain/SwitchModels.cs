namespace AGSwitcher.Domain;

public enum Capability
{
    Unknown,
    Unsupported,
    Supported
}

public enum AgentActivity
{
    Unknown,
    Idle,
    Busy
}

public enum SwitchPhase
{
    Locked,
    Confirming,
    Precheck,
    Switching,
    Verifying,
    Completed,
    Blocked,
    Failed
}

public enum SwitchIssue
{
    None,
    Cancelled,
    ConcurrentOperation,
    InvalidConfirmation,
    NoConfirmation,
    UnknownTarget,
    ClientMissing,
    CompatibilityUnknown,
    IncompatibleVersion,
    ClientStateUnknown,
    ClientNotRunning,
    ManagedInstanceConflict,
    UnfinishedOperation,
    AgentBusy,
    AgentStateUnknown,
    ActiveIdentityUnknown,
    AlreadyActive,
    RecoveryRequired,
    SwitchFailed,
    RestartFailed,
    AuthUnavailable,
    VerificationFailed,
    UnexpectedFailure
}

public enum SwitchAttempt
{
    Applied,
    Failed,
    RestartFailed,
    AuthUnavailable
}

public readonly record struct SwitchResult(SwitchPhase Phase, SwitchIssue Issue = SwitchIssue.None);
