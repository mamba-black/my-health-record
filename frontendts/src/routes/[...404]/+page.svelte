<script lang="ts">
import { base } from "$app/paths";
import svelteLogo from "$lib/svelte.svg";
</script>

<!-- =============================================================== -->
<hr/>
<center>
  <table>
    <tbody>
    <tr>
      <td><a href="{base}/search"><img alt="Vite Logo" class="logo" src="{base}/vite.svg"/></a></td>
      <td>
        <img alt="Svelte Logo" class="logo svelte" src={svelteLogo}/>
      </td>
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
