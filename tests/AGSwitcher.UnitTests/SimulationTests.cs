using AGSwitcher.Application;
using AGSwitcher.Domain;
using AGSwitcher.Infrastructure.Simulation;

namespace AGSwitcher.UnitTests;

public sealed class SimulationTests
{
    [Theory]
    [InlineData(SimulationScenario.AgentBusy, SwitchIssue.AgentBusy)]
    [InlineData(SimulationScenario.AgentStateUnknown, SwitchIssue.AgentStateUnknown)]
    [InlineData(SimulationScenario.ClientMissing, SwitchIssue.ClientMissing)]
    [InlineData(SimulationScenario.ClientStopped, SwitchIssue.ClientNotRunning)]
    [InlineData(SimulationScenario.IncompatibleVersion, SwitchIssue.IncompatibleVersion)]
    [InlineData(SimulationScenario.CompatibilityUnknown, SwitchIssue.CompatibilityUnknown)]
    [InlineData(SimulationScenario.OtherManagedInstance, SwitchIssue.ManagedInstanceConflict)]
    [InlineData(SimulationScenario.UnfinishedOperation, SwitchIssue.UnfinishedOperation)]
    [InlineData(SimulationScenario.UnknownIdentity, SwitchIssue.ActiveIdentityUnknown)]
    public async Task UnsafeScenarioIsBlockedAndRemainsLocked(SimulationScenario scenario, SwitchIssue issue)
    {
        var (accounts, client, coordinator) = Create(scenario);
        var previousId = client.Inspect().ActiveAccountId;

        coordinator.Begin(accounts[1].Id);
        var result = await coordinator.ConfirmAsync(coordinator.Status.Challenge!);

        Assert.Equal(new SwitchResult(SwitchPhase.Blocked, issue), result);
        Assert.Equal((SwitchPhase.Locked, null), coordinator.Status);
        Assert.Equal(previousId, client.Inspect().ActiveAccountId);
    }

    [Theory]
    [InlineData(SimulationScenario.SwitchFailure, SwitchIssue.SwitchFailed)]
    [InlineData(SimulationScenario.RestartFailure, SwitchIssue.RestartFailed)]
    [InlineData(SimulationScenario.AuthUnavailable, SwitchIssue.AuthUnavailable)]
    [InlineData(SimulationScenario.VerificationFailure, SwitchIssue.VerificationFailed)]
    public async Task FailureDoesNotCompleteAndRetryWorksAfterExplicitReset(SimulationScenario scenario, SwitchIssue issue)
    {
        var (accounts, client, coordinator) = Create(scenario);
        coordinator.Begin(accounts[1].Id);
        Assert.Equal(new SwitchResult(SwitchPhase.Failed, issue),
            await coordinator.ConfirmAsync(coordinator.Status.Challenge!));
        Assert.Equal((SwitchPhase.Locked, null), coordinator.Status);
        Assert.Equal(SwitchIssue.RecoveryRequired, coordinator.Begin(accounts[1].Id).Issue);

        if (scenario is SimulationScenario.RestartFailure or SimulationScenario.VerificationFailure)
        {
            Assert.Equal(SwitchPhase.Blocked, coordinator.Reconcile().Phase);
            Assert.Equal(SwitchIssue.RecoveryRequired, coordinator.Begin(accounts[1].Id).Issue);
        }

        client.Reset(accounts[0].Id);
        Assert.Equal(SwitchPhase.Locked, coordinator.Reconcile().Phase);
        coordinator.Begin(accounts[1].Id);
        Assert.Equal(SwitchPhase.Completed, (await coordinator.ConfirmAsync(coordinator.Status.Challenge!)).Phase);
        Assert.Equal(accounts[1].Id, client.Inspect().ActiveAccountId);
    }

    private static (IReadOnlyList<Account>, SimulatedClient, SwitchCoordinator) Create(SimulationScenario scenario)
    {
        var catalog = new AccountCatalog();
        var client = new SimulatedClient(catalog.Accounts[0].Id, TimeSpan.Zero) { Scenario = scenario };
        return (catalog.Accounts, client, new SwitchCoordinator(catalog, client, new RecordingAudit()));
    }
}
