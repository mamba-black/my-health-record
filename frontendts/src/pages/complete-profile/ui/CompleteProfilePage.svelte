<script lang="ts">
import { ProfileForm } from "$features/profile";
</script>

<div class="py-6 flex flex-col justify-center sm:py-12">
  <div class="relative py-3 sm:max-w-2xl sm:mx-auto w-full">
    <div
      class="absolute inset-0 bg-gradient-to-r from-cyan-500 to-slate-800 shadow-lg transform -skew-y-3 sm:skew-y-0 sm:-rotate-2 sm:rounded-3xl"
    ></div>
    <div class="relative px-6 py-10 bg-white shadow-lg sm:rounded-3xl sm:p-12">
      <ProfileForm />
    </div>
  </div>
</div>
