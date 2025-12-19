use gpui::actions;

actions!(
    ndownload,
    [
        // Navigation
        GoBack,
        // Channel management
        AddChannel,
        RefreshChannels,
        // Video management
        RefreshVideos,
        // Download management
        CancelDownload,
        // Application
        Quit,
    ]
);
