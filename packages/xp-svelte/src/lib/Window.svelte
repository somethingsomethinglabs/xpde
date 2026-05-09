<script lang="ts">
  export let title = "Window";
</script>

<div class="window">
  <slot name="title">
    <div class="title-bar">
      <div class="title-bar-text">{title}</div>
    </div>
  </slot>
  <div class="window-body">
    <slot />
  </div>
</div>
