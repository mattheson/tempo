import { RefreshCw } from "lucide-react";
import { useState } from "react";

export function RefreshButton({ refresh }: { refresh: () => Promise<any> }) {
  const [refreshing, setRefreshing] = useState<boolean>(false);

  return (
    <RefreshCw
      className={"cursor-pointer" + (refreshing ? "animate-spin" : "")}
      onClick={() => {
        setRefreshing(true);
        refresh().finally(() => {
          setRefreshing(false);
        });
      }}
    />
  );
}
