namespace AGSwitcher.Infrastructure.Desktop;

// Acquire and dispose on the same dispatcher thread for the lifetime of the window.
public sealed class InstanceLease : IDisposable
{
    public const string ApplicationName = @"Local\AGAccountSwitcherLite.Simulation";

    private readonly Mutex _mutex;
    private bool _disposed;

    private InstanceLease(Mutex mutex) => _mutex = mutex;

    public static InstanceLease? TryAcquire(string name)
    {
        var mutex = new Mutex(false, name);
        try
        {
            try
            {
                if (!mutex.WaitOne(0))
                {
                    mutex.Dispose();
                    return null;
                }
            }
            catch (AbandonedMutexException)
            {
                // The caller owns the abandoned mutex. The simulation is recreated from scratch.
            }

            return new InstanceLease(mutex);
        }
        catch
        {
            mutex.Dispose();
            throw;
        }
    }

    public void Dispose()
    {
        if (!_disposed)
        {
            _mutex.ReleaseMutex();
            _mutex.Dispose();
            _disposed = true;
        }
    }
}
