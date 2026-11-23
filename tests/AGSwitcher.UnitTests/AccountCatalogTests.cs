using AGSwitcher.Domain;

namespace AGSwitcher.UnitTests;

public sealed class AccountCatalogTests
{
    [Fact]
    public void FourSeedAccountsAreDistinctAndOnlyNonActiveAccountsCanBeRemoved()
    {
        var catalog = new AccountCatalog();
        var initial = catalog.Accounts.ToArray();
        Assert.Equal(4, initial.Length);
        Assert.Equal(4, initial.Select(account => account.Id).Distinct().Count());

        var added = catalog.Add();
        Assert.Equal(5, catalog.Accounts.Count);
        Assert.Equal("模拟账号 05", added.Label);
        Assert.False(catalog.Remove(initial[0].Id, initial[0].Id));
        Assert.False(catalog.Remove(added.Id, null));
        Assert.True(catalog.Remove(added.Id, initial[0].Id));
        Assert.False(catalog.Contains(added.Id));
        Assert.Equal(initial, catalog.Accounts);
    }
}
