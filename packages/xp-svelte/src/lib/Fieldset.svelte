<script lang="ts">
  export let legend = "";
</script>

<fieldset>
  {#if legend}<legend>{legend}</legend>{/if}
  <slot />
</fieldset>
