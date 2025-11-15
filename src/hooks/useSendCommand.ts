import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export const useSendCommand = () => {
  return useCallback(async (command: string) => {
    try {
      await invoke("send_command", { command });
    } catch (error) {
      console.error("Failed to send command:", error);
    }
  }, []);
};
