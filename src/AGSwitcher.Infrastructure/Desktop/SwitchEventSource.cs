using System.Diagnostics.Tracing;
using AGSwitcher.Application;
using AGSwitcher.Domain;

namespace AGSwitcher.Infrastructure.Desktop;

[EventSource(Name = "AGSwitcher-Simulation")]
internal sealed class SwitchEventSource : EventSource
{
    internal static readonly SwitchEventSource Log = new();

    [Event(1, Level = EventLevel.Informational)]
    public void Transition(int phase, int issue) => WriteEvent(1, phase, issue);
}

public sealed class LocalSwitchAudit(bool enabled) : ISwitchAudit
{
    public void Write(SwitchResult result)
    {
        if (enabled)
        {
            SwitchEventSource.Log.Transition((int)result.Phase, (int)result.Issue);
        }
    }
}
