<script lang="ts">
  export let text = "Title";
</script>

<div class="title-bar">
  <div class="title-bar-text">{text}</div>
  <div class="title-bar-controls">
    <slot />
  </div>
</div>
