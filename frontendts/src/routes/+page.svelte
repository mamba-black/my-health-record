<script lang="ts">
  import svelteLogo from "$lib/svelte.svg";
  import {base} from "$app/paths";
</script>

<!-- =============================================================== -->
<hr/>
<center>
  <table>
    <tbody>
    <tr>
      <td><a href="{base}/search"><img alt="Vite Logo" class="logo" src="{base}/vite.svg"/></a></td>
      <td><a href="{base}/search"><img alt="Svelte Logo" class="logo svelte" src={svelteLogo}/></a></td>
    </tr>
    </tbody>
  </table>
</center>
<!-- =============================================================== -->
<style>
  .logo {
    height: 6em;
    padding: 1.5em;
    will-change: filter;
    transition: filter 300ms;
  }

  .logo:hover {
    filter: drop-shadow(0 0 2em #646cffaa);
  }

  .logo.svelte:hover {
    filter: drop-shadow(0 0 2em #ff3e00aa);
  }
</style>
