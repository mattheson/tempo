import { fatal, scanPlugins } from "./commands";
import { Card } from "./components/ui/card";
import { SettingsForm } from "./SettingsForm";

export function Setup() {
  return (
    <div className="p-6 bg-inherit">
      <h1 className="text-4xl mb-5 font-bold">Setup</h1>
      <b>Welcome!</b>
      <br />
      <br />
      <p>Thank you for taking some time to try out Tempo!</p>
      <br />
      <p>Please note that this is a prototype version.</p>
      <b className="text-red-500">
        Do not use Tempo to store any valuable work yet! Always create separate
        copies of your projects.
      </b>
      <br />
      <br />
      <Card className="p-5">
        <h2 className="text-2xl mb-5 font-bold">Settings</h2>
        Please enter a default username.
        <br />
        This username is used to identify yourself in folders.
        <br />
        You can optionally set other settings here as well.
        <SettingsForm
          doOnce={{
            cb: () => {
              return scanPlugins().catch((err) => {
                fatal(
                  "Tempo failed to perform an initial scan of your plugins! Please report this bug. Error: " +
                    JSON.stringify(err)
                );
              });
            },
            loadingMsg: "Performing initial plugin scan...",
          }}
        />
      </Card>
    </div>
  );
}
