import { useState, useEffect } from "react";
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  FormControlLabel,
  Checkbox,
  TextField,
  Box,
} from "@mui/material";
import { invoke } from "@tauri-apps/api/core";

interface Settings {
  word_wrap_enabled: boolean;
  word_wrap_length: number;
}

interface SettingsDialogProps {
  open: boolean;
  onClose: () => void;
}

export default function SettingsDialog({ open, onClose }: SettingsDialogProps) {
  const [settings, setSettings] = useState<Settings>({
    word_wrap_enabled: true,
    word_wrap_length: 100,
  });

  const [wrapLengthInput, setWrapLengthInput] = useState<string>("100");

  useEffect(() => {
    if (open) {
      loadSettings();
    }
  }, [open]);

  const loadSettings = async () => {
    try {
      const loadedSettings = await invoke<Settings>("get_settings");
      setSettings(loadedSettings);
      setWrapLengthInput(loadedSettings.word_wrap_length.toString());
    } catch (error) {
      console.error("Failed to load settings:", error);
    }
  };

  const validateLength = (input: string): number => {
    const min = 20;
    const max = 200;
    const parsed = parseInt(input);

    if (isNaN(parsed) || parsed < min) {
      return min;
    }
    if (parsed > max) {
      return max;
    }
    return parsed;
  };

  const handleSave = async () => {
    const finalLength = validateLength(wrapLengthInput);
    try {
      await invoke("save_settings", {
        settings: {
          ...settings,
          word_wrap_length: finalLength,
        },
      });
      onClose();
    } catch (error) {
      console.error("Failed to save settings:", error);
    }
  };

  const handleCancel = () => {
    onClose();
  };

  const handleBlur = () => {
    const validatedLength = validateLength(wrapLengthInput);
    setSettings((prev) => ({
      ...prev,
      word_wrap_length: validatedLength,
    }));
    setWrapLengthInput(validatedLength.toString());
  };

  return (
    <Dialog open={open} onClose={handleCancel} maxWidth="sm" fullWidth>
      <DialogTitle>Settings</DialogTitle>
      <DialogContent>
        <Box sx={{ pt: 2 }}>
          <FormControlLabel
            control={
              <Checkbox
                checked={settings.word_wrap_enabled}
                onChange={(e) =>
                  setSettings({
                    ...settings,
                    word_wrap_enabled: e.target.checked,
                  })
                }
              />
            }
            label="Enable Word Wrap"
          />
          <TextField
            fullWidth
            label="Word Wrap Length"
            type="number" // Retain type="number" for mobile keyboards
            value={wrapLengthInput}
            onChange={(e) => setWrapLengthInput(e.target.value)}
            onBlur={handleBlur}
            disabled={!settings.word_wrap_enabled}
            sx={{
              mt: 2,
              // hide the number input spin buttons
              "& input[type=number]::-webkit-outer-spin-button, & input[type=number]::-webkit-inner-spin-button":
                {
                  "-webkit-appearance": "none",
                  margin: 0,
                },
              "& input[type=number]": {
                "-moz-appearance": "textfield",
              },
            }}
            slotProps={{
              htmlInput: {
                min: 20,
                max: 200,
              },
            }}
            helperText={`Current saved length: ${settings.word_wrap_length}. Enter value between 20-200.`}
          />
        </Box>
      </DialogContent>
      <DialogActions>
        <Button onClick={handleCancel}>Cancel</Button>
        <Button onClick={handleSave} variant="contained">
          Save
        </Button>
      </DialogActions>
    </Dialog>
  );
}
