<script>
  import { Button } from "flowbite-svelte";
  import { Download } from "lucide-svelte";
</script>

<title>Tempo Download</title>

<h1 class="text-4xl flex px-10 pb-10">Download</h1>
<div class="px-10">
  <p>
    <b>Warning:</b> Tempo is still in an early prototype phase. Expect that everything
    might be broken! It works on my computer but I can't guarantee it will on yours...
  </p>
  <p>
    Join the <a
      href="https://discord.gg/u4WWJ4td5D"
      class="text-blue-600 dark:text-blue-500 hover:underline">Discord</a
    > to share feedback/bug reports/etc! All feedback is greatly appreciated!
  </p>
  <br />
  You can try Tempo now to see how it fits into your workflow by yourself or with
  your collaborators. Tempo has lots of missing features and lots of bugs.
  <p><b>Be sure to create separate copies of your projects!</b></p>
</div>

<div class="flex flex-col py-12 px-10">
  <Button class="bg-orange-400 my-2 shadow-md">
    <div class="flex flex-row gap-x-3 items-center">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="3em"
        height="3em"
        viewBox="0 0 24 24"
        {...$$props}
      >
        <path
          fill="currentColor"
          d="M12.152 6.896c-.948 0-2.415-1.078-3.96-1.04c-2.04.027-3.91 1.183-4.961 3.014c-2.117 3.675-.546 9.103 1.519 12.09c1.013 1.454 2.208 3.09 3.792 3.039c1.52-.065 2.09-.987 3.935-.987c1.831 0 2.35.987 3.96.948c1.637-.026 2.676-1.48 3.676-2.948c1.156-1.688 1.636-3.325 1.662-3.415c-.039-.013-3.182-1.221-3.22-4.857c-.026-3.04 2.48-4.494 2.597-4.559c-1.429-2.09-3.623-2.324-4.39-2.376c-2-.156-3.675 1.09-4.61 1.09zM15.53 3.83c.843-1.012 1.4-2.427 1.245-3.83c-1.207.052-2.662.805-3.532 1.818c-.78.896-1.454 2.338-1.273 3.714c1.338.104 2.715-.688 3.559-1.701"
        />
      </svg>
      macOS
      <!-- <i class="ml-2 font-light">v0.0.0</i> -->
    </div>
    <div class="ml-auto">
      <Download />
    </div>
  </Button>
  <Button class="bg-orange-400 my-2 shadow-md" disabled>
    <div class="flex flex-row gap-x-3 items-center">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="3em"
        height="3em"
        viewBox="0 0 32 32"
        {...$$props}
      >
        <path
          fill="currentColor"
          d="m0 4.599l13-1.796v12.599H0zm14.599-2L32 0v15.197H14.599zM0 16.803h13v12.599L0 27.599zm14.599 0H32V32l-17.197-2.401z"
        />
      </svg>
      Windows
    </div>
    <div class="ml-auto">
      <p>Coming soon...</p>
    </div>
  </Button>
</div>
