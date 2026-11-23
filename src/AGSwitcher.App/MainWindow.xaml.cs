using System.ComponentModel;
using System.Windows;

namespace AGSwitcher.App;

public partial class MainWindow : Window
{
    public MainWindow() => InitializeComponent();

    protected override void OnClosing(CancelEventArgs e)
    {
        if (DataContext is SwitchViewModel { CanClose: false })
        {
            e.Cancel = true;
        }

        base.OnClosing(e);
    }

    private void OnConfirmationVisibilityChanged(object sender, DependencyPropertyChangedEventArgs e)
    {
        if (e.NewValue is true)
        {
            Dispatcher.InvokeAsync(() => ConfirmationBox.Focus());
        }
    }
}
