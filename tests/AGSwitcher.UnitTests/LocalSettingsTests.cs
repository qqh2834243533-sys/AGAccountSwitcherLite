using AGSwitcher.Infrastructure.Desktop;

namespace AGSwitcher.UnitTests;

public sealed class LocalSettingsTests
{
    [Theory]
    [InlineData("""{"EnableDiagnostics":true}""", true, false)]
    [InlineData("""{"EnableDiagnostics":false}""", false, false)]
    [InlineData("""{"extraField":true}""", false, true)]
    [InlineData("""{"EnableDiagnostics":"text"}""", false, true)]
    [InlineData("null", false, true)]
    [InlineData("", false, true)]
    public void SettingsAcceptOnlyTheDocumentedBoolean(string json, bool enabled, bool fallback)
    {
        var file = Path.GetTempFileName();
        try
        {
            File.WriteAllText(file, json);
            var result = LocalSettingsStore.Load(file);
            Assert.Equal(enabled, result.Settings.EnableDiagnostics);
            Assert.Equal(fallback, result.UsedFallback);
        }
        finally
        {
            File.Delete(file);
        }
    }

    [Fact]
    public void AbsentSettingsDisableDiagnosticsByDefault()
    {
        var file = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString(), "settings.json");
        Assert.Equal((new LocalSettings(), false), LocalSettingsStore.Load(file));
    }
}
