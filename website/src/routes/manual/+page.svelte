<script lang="ts">
  import Toc from "svelte-toc";
  import {
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
    Checkbox,
    TableSearch,
  } from "flowbite-svelte";
  import {
    ChevronDown,
    ChevronRight,
    Folder,
    GitBranch,
    Paperclip,
    Reply,
    Send,
  } from "lucide-svelte";

  import { onMount } from "svelte";
  import { tick } from "svelte";
  import { base } from "$app/paths";

  let showToc = false;

  onMount(async () => {
    await tick();
    showToc = true;
  });
</script>

<title>Tempo Manual</title>

<div class="flex max-w-full mb-16">
  <main class="px-10 leading-7 flex-1 max-w-full">
    <h1 class="text-4xl pb-8">Manual</h1>
    <h2 class="text-2xl font-bold">Setup</h2>
    <p>
      To get started with Tempo, navigate to the <a
        href={base + "/download"}
        class="text-blue-600 dark:text-blue-500 hover:underline">Download</a
      > page.
    </p>
    <h3 class="text-xl font-semibold mt-4">macOS</h3>
    <p>
      You might have to go into macOS's <b>Privacy & Security</b> settings in
      System Settings to launch Tempo. Tempo also requires
      <b>Full Disk Access</b>
      on macOS, which is used to simplify the process of scanning plugins and copying
      referenced files into folders. You will be prompted to grant Full Disk Access
      on Tempo's first startup.
    </p>
    <h3 class="text-xl font-semibold mt-4">Windows</h3>
    <p><i>Windows support is coming soon!</i></p>
    <h2 class="text-2xl font-bold mt-4">Usage</h2>
    <h3 class="text-xl font-semibold mt-4">Creating Folders</h3>
    <p>
      Tempo operates on <b>folders</b>, which can be used locally and/or shared
      with other users.
      <i>You can think of a folder like a Discord server</i>.
    </p>
    <p class="mt-4">
      To create or add an existing folder, click the <b class="text-2xl">+</b>
      button next to "Folders" on the home page. You will be prompted to select a
      folder.
    </p>
    <p class="mt-4">
      When creating a folder, Tempo will create a <code>tempo</code> directory inside
      of the provided folder. This folder holds all of Tempo's data, including any
      project files, audio files or other files referenced in projects. Refrain from
      tampering with the contents of this directory, as you might accidentally corrupt
      your folder.
    </p>
    <h3 class="text-xl font-semibold mt-4">Adding Notes</h3>
    <p>
      Within folders, you can create <b>notes</b>. Notes are similar to messages
      that you'd send in apps like Discord or iMessage. You can optionally add
      attachments to your notes, which is covered in the following section.
    </p>
    <p class="mt-4">
      <i
        >Note that notes will not show up instantly on your collaborators'
        screen!</i
      > Tempo's latency is based on the speed of your file sync service.
    </p>
    <p class="mt-4">
      To create a note, simply type in Tempo's chat box and press the <Send
        class="inline"
      /> button. Writing a note body is optional if you've added an attachment.
    </p>
    <p class="mt-4">
      A note can be a reply to a previous note. Press the <Reply
        class="inline"
      /> button at the top right of a previous note to reply to it. Replying is how
      Tempo tries to address version management. Reference the Version Management
      section for more details.
    </p>
    <h3 class="text-xl font-semibold mt-4">Attachments</h3>
    <p>
      Since Tempo aims to help you manage your work as a music producer,
      attachments are a feature which you should use quite often.
    </p>
    <p class="mt-4">
      You can <b>drag and drop</b> a file in the note input box to attach it to
      a note, or press the <Paperclip class="inline" /> button to select a file.
    </p>
    <h4 class="text-xl font-semibold mt-4">Supported Attachments</h4>
    <h5 class="text-lg mt-4">
      ∙ Ableton Live Sets (<code>.als</code>)
    </h5>
    <p>
      You can attach an Ableton Live Set/project file (an <code>.als</code>
      file) file to a note.
      <b
        >Tempo automatically handles copying any referenced files in the project</b
      >, removing any need for you to "Collect All and Save".
    </p>
    <p class="mt-4">
      Tempo gives you information about whether it was <b
        >able to find all files referenced in the project</b
      >, and whether your project
      <b
        >contains any plugins which your or your collaborators don't have
        installed</b
      >. You can use this information to fix missing samples, freeze/flatten
      tracks, or adjust your project as needed. Alternatively, you can ignore
      any warnings.
    </p>
    <p class="mt-4">
      You can also add a <b>rendered version</b> of your project to the project attachment.
      This enables you and your collaborators to use Tempo's audio player to quickly
      listen to your project.
    </p>
    <h5 class="text-lg mt-4">
      ∙ Audio Files (<code>.mp3/.wav/.flac/etc.</code>)
    </h5>
    <p>You can also attach audio files to notes.</p>
    <h4 class="text-xl font-semibold mt-4">Titles</h4>
    <p>
      Live Set attachments and audio file attachments provide the ability for
      you to write a title. This title is optional for audio files. <i
        class="text-xs"
        >(TODO: Titles could probably be optional for Ableton projects as well!
        They are required on projects as of now.)</i
      >
      Titles are an opportunity for you to write a short description about your attachment.
      If you're using Tempo to manage versions of an Ableton project, you might use
      the title to summarize any changes you've made to the project.
    </p>
    <h3 class="text-xl font-semibold mt-4">Channels</h3>
    <p>
      Like Discord, you can create <b>channels</b> within folders. Channels can be
      used to further organize your work in Tempo.
    </p>
    <p class="mt-4">
      Press the channel dropdown in Tempo's top bar to select or create a
      channel.
    </p>
    <h3 class="text-xl font-semibold mt-4">Version Management</h3>
    <p>Tempo aims to help you manage different versions of your tracks.</p>
    <p class="mt-4">
      Say you create a rough draft of a song and you want to pass it off to your
      collaborators, so you attach it to a note and add it to a shared folder.
    </p>
    <p>
      Using Tempo, your collaborators can save a <b>local copy</b> of your project
      and make their desired changes.
    </p>
    <p>
      When your collaborators are done with their new versions, they can create
      new notes (with their versions attached), and add them in the folder as a
      <b>reply to your note containing your original project.</b>
    </p>

    <p class="mt-4">
      Now, imagine that you and your collaborators repeat this process dozens of
      times. You'll end up with lots of notes, and it can be difficult to tell
      the relationships between them. To address this, Tempo provides a Tree
      View (accessible by pressing the <GitBranch class="inline" /> button), which
      allows you to clearly see the reply relationships between any notes with attachments.
    </p>

    <h3 class="text-xl font-semibold mt-4">Writing Feedback</h3>
    <p>You can use Tempo to write feedback or take notes on your work.</p>
    <p>Tempo provides <b>commenting</b> on notes with attachments.</p>

    <h2 class="text-2xl font-bold mt-4">Sharing Folders</h2>
    <p>
      Tempo can be used without sharing your folders with your collaborators. If
      you'd like to share folders with collaborators, you will need to set up a <b
        >file sync service</b
      > and share a directory with your collaborators. Tempo is designed to work
      well with any sync service, but many haven't been tested and could behave unexpectedly.
    </p>
    <h3 class="text-xl font-semibold mt-4">Sync Services</h3>
    <p>
      Carefully consider whether a <b>peer-to-peer</b> or <b>cloud-based</b> file
      sync service would work best for you. With peer-to-peer services, you and your
      collaborators need to have your computers turned on at the same time in order
      for your work to synchronize. This can be inconvenient if you or your collaborators
      use laptops. Setting up a constantly-running computer can remedy this. Cloud-based
      sync services avoid this issue altogether.
    </p>
    <p class="mt-4">
      <b>Recommendation:</b> If you're using a cloud-based file sync service,
      enable settings to make sure that the sync service
      <i>always downloads local copies</i>. Very often, cloud-based sync
      services won't download local copies of files until they're opened, which
      can slow down Tempo.
    </p>
    <p class="mt-4">
      <i>
        Reference
        <a
          href="https://en.wikipedia.org/wiki/Comparison_of_file_synchronization_software"
          class="text-blue-600 dark:text-blue-500 hover:underline">Wikipedia</a
        > for more file sync services.
      </i>
    </p>
    <!-- this table is not very mobile friendly but oh well -->
    <Table divClass="mt-2 z-[-100] relative">
      <TableHead class="bg-gray-300">
        <TableHeadCell>Name</TableHeadCell>
        <TableHeadCell>Tested?</TableHeadCell>
        <TableHeadCell>Notes</TableHeadCell>
        <TableHeadCell>Cost</TableHeadCell>
      </TableHead>
      <TableBody tableBodyClass="divide-y w-fit relative">
        <TableBodyRow>
          <TableBodyCell>
            <a
              href="https://syncthing.net/"
              class="text-blue-600 dark:text-blue-500 hover:underline"
              >Syncthing</a
            >
          </TableBodyCell>
          <TableBodyCell><p class="text-xl">❓</p></TableBodyCell>
          <TableBodyCell><i>Peer-to-peer</i></TableBodyCell>
          <TableBodyCell>Free</TableBodyCell>
        </TableBodyRow>
        <TableBodyRow>
          <TableBodyCell>
            <a
              href="https://www.resilio.com/sync/"
              class="text-blue-600 dark:text-blue-500 hover:underline"
              >Resilio Sync</a
            >
          </TableBodyCell>
          <TableBodyCell><p class="text-xl">❌</p></TableBodyCell>
          <TableBodyCell><i>Peer-to-peer</i></TableBodyCell>
          <TableBodyCell>Free, extra features with pro version</TableBodyCell>
        </TableBodyRow>
        <TableBodyRow>
          <TableBodyCell>iCloud Drive</TableBodyCell>
          <TableBodyCell><p class="text-xl">❓</p></TableBodyCell>
          <TableBodyCell><i>Cloud-based, macOS only</i></TableBodyCell>
          <TableBodyCell>5GB free, iCloud+ subscription for more</TableBodyCell>
        </TableBodyRow>
        <TableBodyRow>
          <TableBodyCell>Dropbox</TableBodyCell>
          <TableBodyCell><p class="text-xl">❌</p></TableBodyCell>
          <TableBodyCell><i>Cloud-based</i></TableBodyCell>
          <TableBodyCell>2GB free, paid for more</TableBodyCell>
        </TableBodyRow>
        <TableBodyRow>
          <TableBodyCell>Google Drive</TableBodyCell>
          <TableBodyCell><p class="text-xl">❌</p></TableBodyCell>
          <TableBodyCell><i>Cloud-based</i></TableBodyCell>
          <TableBodyCell>15GB free, paid for more</TableBodyCell>
        </TableBodyRow>
      </TableBody>
      <tfoot>
        <tr
          ><td>
            <div class="flex gap-2">
              <span>✅ = extensively tested</span>
              <span>❓ = limited testing</span>
              <span>❌ = untested</span>
            </div>
          </td></tr
        >
      </tfoot>
    </Table>
    <h3 class="text-xl font-semibold mt-4">Sharing a Folder</h3>
    <p>
      Once you and your collaborators have set up your sync service of choice,
      you will need to <b>share a directory</b> between you and all of your collaborators.
    </p>
    <p class="mt-4">
      When you have a directory shared, you can begin creating your Tempo
      folder(s). I recommend creating your Tempo folders as <b>subdirectories</b
      >
      rather than directly using the shared directory, since this will allow you
      to create multiple Tempo folders in the directory. An illustration is provided
      below:
    </p>
    <div class="m-4 bg-white rounded-lg p-2">
      <ChevronDown class="inline" />
      <i
        ><Folder class="inline mr-2" />(your directory shared with a sync
        service)</i
      >
      <div class="ml-16">
        <ChevronDown class="inline" /><Folder class="inline mr-2" />my tempo
        folder
        <div class="ml-16">
          <ChevronRight class="inline" /><Folder class="inline mr-2" /><code
            >tempo</code
          > (tempo app data)
        </div>
      </div>
      <div class="ml-16">
        <ChevronDown class="inline" /><Folder class="inline mr-2" />my other
        tempo folder
        <div class="ml-16">
          <ChevronRight class="inline" /><Folder class="inline mr-2" /><code
            >tempo</code
          >
        </div>
      </div>
    </div>
    <p class="mt-4">
      Once you've created your Tempo folder(s), your collaborators should be
      able to hit the <b class="text-2xl">+</b> button to add them to Tempo! In the
      example above, your collaborators would add the "my tempo folder" and "my other
      tempo folder" directories to Tempo.
    </p>
    <h2 class="text-2xl font-bold mt-4">Limitations</h2>
    <p>
      Tempo aims to enhance your workflow with Ableton. However, Tempo is
      completely third-party software. With this comes many limitations.
    </p>
    <h3 class="text-xl font-semibold mt-4">Project Files/Plugin Databases</h3>
    <p>
      Tempo is able to parse bits and pieces of <b
        >Ableton's project file format</b
      >
      and <b>Ableton's plugin database</b> to provide its features. Neither the project
      file or plugin database format are officially documented by Ableton. If Ableton
      decides to make significant changes to either format, this will probably break
      Tempo.
    </p>
    <p class="mt-4">
      It's possible that Tempo improperly parses some parts of Ableton's project
      file format.
    </p>
    <p class="mt-4">
      <b>Tempo does not support very old versions of Ableton.</b> Tempo should work
      with later versions of Live 10 (I think, haven't tested). Supporting old versions
      of Ableton is not a priority for Tempo.
    </p>
    <h3 class="text-xl font-semibold mt-4">Support for Other DAWs</h3>
    <p>
      <b>Tempo will not support other DAWs</b> until a comprehensive Rust library
      for parsing DAW project files is developed. I'm interested in working on this
      library and will update this section with more details in the future.
    </p>
    <h3 class="text-xl font-semibold mt-4">Mutability</h3>
    <p>
      Currently, you are not able to edit the content of notes or delete them,
      and you also can't delete or rename channels. These features will be
      implemented at some point.
    </p>
    <h2 class="text-2xl font-bold mt-4">Reporting Bugs</h2>
    <p>
      Please join the Tempo Discord or post an issue on GitHub if you encounter
      a bug. <b
        >Please provide info about your OS version and Ableton version as well.</b
      > Your feedback is greatly appreciated!
    </p>
  </main>
  {#if showToc}
    <div class="flex-none min-w-4">
      <Toc --toc-active-bg="orange" --toc-desktop-sticky-top="8em" />
    </div>
  {/if}
</div>

<style>
  h1,
  h2,
  h3,
  h4,
  h5 {
    scroll-margin-top: 4em;
  }
</style>
