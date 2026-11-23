using System.Text.Json;
using System.Text.Json.Serialization;

namespace AGSwitcher.Infrastructure.Desktop;

public sealed record LocalSettings(bool EnableDiagnostics = false);

[JsonSerializable(typeof(LocalSettings))]
[JsonSourceGenerationOptions(UnmappedMemberHandling = JsonUnmappedMemberHandling.Disallow)]
internal sealed partial class SettingsJsonContext : JsonSerializerContext;

public static class LocalSettingsStore
{
    public static string DefaultPath => Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
        "AGAccountSwitcherLite", "settings.json");

    public static (LocalSettings Settings, bool UsedFallback) Load(string path)
    {
        try
        {
            using var stream = File.OpenRead(path);
            if (stream.Length is 0 or > 4096)
            {
                return (new(), true);
            }

            var settings = JsonSerializer.Deserialize(stream, SettingsJsonContext.Default.LocalSettings);
            return settings is null ? (new(), true) : (settings, false);
        }
        catch (FileNotFoundException)
        {
            return (new(), false);
        }
        catch (DirectoryNotFoundException)
        {
            return (new(), false);
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or JsonException)
        {
            return (new(), true);
        }
    }
}
