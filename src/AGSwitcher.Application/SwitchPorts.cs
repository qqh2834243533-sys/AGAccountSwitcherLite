using AGSwitcher.Domain;

namespace AGSwitcher.Application;

public readonly record struct ClientSnapshot(
    bool? Installed,
    Capability Compatibility,
    bool? Running,
    int? ManagedInstanceCount,
    bool? UnfinishedOperation,
    AgentActivity Agent,
    Guid? ActiveAccountId);

public interface ISwitchClient
{
    ClientSnapshot Inspect();
    Task<SwitchAttempt> SwitchAsync(Guid targetId);
}

public interface ISwitchAudit
{
    void Write(SwitchResult result);
}
