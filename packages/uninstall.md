# Uninstall

Run `elarydil-kvm startup disable --component agent` and, when installed,
`elarydil-kvm startup disable --component gui` **as the affected user**. Then run
`elarydil-kvm agent stop` and quit the GUI before uninstalling binaries.

On Debian remove the package through the normal package manager. On other platforms
remove the application/CLI directory after unregistering startup. User configuration
and event history are preserved. Delete the documented per-user directory only if
you explicitly want to erase your settings and captures. Public OS installer lifecycle
hooks and Windows/macOS clean-machine uninstall verification are outstanding.
