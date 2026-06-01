(() => {
    const darkThemes = ['ayu', 'navy', 'coal'];
    const lightThemes = ['light', 'rust'];

    const classList = document.getElementsByTagName('html')[0].classList;

    let isDark = false;
    for (const cssClass of classList) {
        if (darkThemes.includes(cssClass)) {
            isDark = true;
            break;
        }
    }

    const darkVars = {
        background: '#1f2630',
        primaryColor: '#2c3a52',
        primaryTextColor: '#e6edf3',
        primaryBorderColor: '#5b8def',
        secondaryColor: '#3a2c52',
        tertiaryColor: '#1a2230',
        lineColor: '#7aa2f7',
        textColor: '#e6edf3',
        mainBkg: '#2c3a52',
        nodeBorder: '#5b8def',
        clusterBkg: '#243043',
        clusterBorder: '#3d4f6f',
        titleColor: '#e6edf3',
        edgeLabelBackground: '#1f2630',
        fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
        fontSize: '14px',
    };

    const lightVars = {
        background: '#ffffff',
        primaryColor: '#dde7f3',
        primaryTextColor: '#1a1a1a',
        primaryBorderColor: '#3b6fc4',
        secondaryColor: '#e8def0',
        tertiaryColor: '#f5f7fa',
        lineColor: '#3b6fc4',
        textColor: '#1a1a1a',
        mainBkg: '#dde7f3',
        nodeBorder: '#3b6fc4',
        clusterBkg: '#eef2f8',
        clusterBorder: '#a8b9d1',
        titleColor: '#1a1a1a',
        edgeLabelBackground: '#ffffff',
        fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
        fontSize: '14px',
    };

    mermaid.initialize({
        startOnLoad: true,
        theme: 'base',
        themeVariables: isDark ? darkVars : lightVars,
        flowchart: { curve: 'linear', padding: 12, nodeSpacing: 40, rankSpacing: 50 },
        sequence: { diagramMarginX: 20, diagramMarginY: 20, actorMargin: 60, messageMargin: 40 },
    });

    for (const darkTheme of darkThemes) {
        document.getElementById(darkTheme)?.addEventListener('click', () => {
            if (!isDark) window.location.reload();
        });
    }
    for (const lightTheme of lightThemes) {
        document.getElementById(lightTheme)?.addEventListener('click', () => {
            if (isDark) window.location.reload();
        });
    }
})();
