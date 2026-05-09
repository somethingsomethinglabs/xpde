<script>
  const probeExample =
    "busctl --user call org.xpde.Shell /org/xpde/Web1 org.xpde.Web1 ProbeUrl s 'https://example.com'";
  const pinExample = [
    'busctl --user call org.xpde.Shell /org/xpde/Web1 org.xpde.Web1 PinSite s ',
    "'{\"url\":\"https://example.com\",\"include_sitemap\":true,\"sitemap_max_depth\":2,\"include_feeds\":true}'",
  ].join("");
</script>

<div class="window">
  <div class="title-bar">
    <div class="title-bar-text">Pin Web App Wizard</div>
  </div>
  <div class="window-body">
    <p>
      The install wizard UI will call <code>ProbeUrl</code> and <code>PinSite</code> on
      <code>org.xpde.Web1</code>. From the command line you can probe and pin:
    </p>
    <pre><code>{probeExample}

{pinExample}</code></pre>
    <p>
      This writes <code>~/.config/xpde/web-objects/&lt;id&gt;.toml</code> and
      <code>~/.local/share/applications/xpde-web-&lt;id&gt;.desktop</code>.
    </p>
  </div>
</div>
