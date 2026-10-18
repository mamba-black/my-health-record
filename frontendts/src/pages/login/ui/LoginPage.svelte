<svelte:head>
  <meta
    content="937186309482-kmain19anvo9rjljci5c7d6iiibuhvlu.apps.googleusercontent.com"
    name="google-signin-client_id"
  />
</svelte:head>

<script lang="ts">
import { LoginForm } from "$features/auth";
</script>

<div class="py-6 flex flex-col justify-center sm:py-12">
  <div class="relative py-3 sm:max-w-xl sm:mx-auto">
    <div
      class="absolute inset-0 bg-gradient-to-r from-gray-400 to-gray-900 shadow-lg transform -skew-y-6 sm:skew-y-0 sm:-rotate-6 sm:rounded-3xl"
    ></div>
    <div class="relative px-4 py-10 bg-white shadow-lg sm:rounded-3xl sm:p-20">
      <LoginForm/>
    </div>
  </div>
</div>
