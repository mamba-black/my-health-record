<script lang="ts">
  import viteLogo from "/vite.svg";
  import svelteLogo from "./assets/svelte.svg";
  import Search from "./lib/Search.svelte";
</script>

<Search />

<!-- =============================================================== -->
<hr />
<center>
  <table>
    <tbody>
      <tr>
        <td><img src={viteLogo} class="logo" alt="Vite Logo" /></td>
        <td>
          <img src={svelteLogo} class="logo svelte" alt="Svelte Logo" />
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
