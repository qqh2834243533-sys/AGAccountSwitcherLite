namespace AGSwitcher.Domain;

public sealed record Account(Guid Id, string Label);

public sealed class AccountCatalog
{
    private readonly List<Account> _accounts = [];
    private int _nextNumber;

    public AccountCatalog(int initialCount = 4)
    {
        ArgumentOutOfRangeException.ThrowIfNegative(initialCount);
        for (int index = 0; index < initialCount; index++)
        {
            Add();
        }
    }

    public IReadOnlyList<Account> Accounts => _accounts.AsReadOnly();

    public Account Add()
    {
        var account = new Account(Guid.NewGuid(), $"模拟账号 {++_nextNumber:D2}");
        _accounts.Add(account);
        return account;
    }

    public bool Contains(Guid id) => _accounts.Exists(account => account.Id == id);

    public bool Remove(Guid id, Guid? activeId) =>
        activeId is not null && activeId != id && _accounts.RemoveAll(account => account.Id == id) == 1;
}
