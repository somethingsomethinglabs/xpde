<div class="window">
  <div class="title-bar">
    <div class="title-bar-text">Pinned Web Apps</div>
  </div>
  <div class="window-body">
    <p>
      Pinned sites are stored under <code>~/.config/xpde/web-objects/</code> and exposed on the
      session bus as <code>org.xpde.Web1</code> (see <code>docs/web-objects.md</code>).
    </p>
    <p>Query pinned sites from a terminal:</p>
    <pre>busctl --user call org.xpde.Shell /org/xpde/Web1 org.xpde.Web1 ListPinned</pre>
  </div>
</div>
