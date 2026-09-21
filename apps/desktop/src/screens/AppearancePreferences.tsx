import { Moon, Sun } from 'lucide-react';
import type { CSSProperties } from 'react';
import {
  DEFAULT_WORKSPACE_THEME,
  WORKSPACE_THEMES,
  isTheme,
  isWorkspaceTheme,
  workspacePalette,
  type Theme,
  type WorkspaceTheme,
} from '@clipmill/tokens';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';

export interface AppearancePreferencesProps {
  readonly theme: Theme;
  readonly onThemeChange: (theme: Theme) => void;
  readonly workspaceTheme?: WorkspaceTheme;
  readonly onWorkspaceThemeChange?: (theme: WorkspaceTheme) => void;
}

export function AppearancePreferences({
  theme,
  onThemeChange,
  workspaceTheme = DEFAULT_WORKSPACE_THEME,
  onWorkspaceThemeChange,
}: AppearancePreferencesProps) {
  return (
    <div className="appearance-preferences">
      <div className="appearance-mode-row">
        <div>
          <h3>Color mode</h3>
          <p>Light or dark, in any theme.</p>
        </div>
        <RadioGroup
          value={theme}
          onValueChange={(value) => {
            if (isTheme(value)) onThemeChange(value);
          }}
          aria-label="Color appearance"
          className="appearance-mode-options"
        >
          {(['light', 'dark'] as const).map((option) => (
            <label key={option} htmlFor={`appearance-${option}`}>
              {option === 'light' ? <Sun aria-hidden="true" /> : <Moon aria-hidden="true" />}
              {option === 'light' ? 'Light' : 'Dark'}
              <RadioGroupItem
                id={`appearance-${option}`}
                value={option}
                aria-label={option === 'light' ? 'Light appearance' : 'Dark appearance'}
              />
            </label>
          ))}
        </RadioGroup>
      </div>
      {onWorkspaceThemeChange && (
        <div>
          <div className="appearance-theme-heading">
            <h3>Workspace theme</h3>
            <span>DM Sans + IBM Plex Mono</span>
          </div>
          <RadioGroup
            value={workspaceTheme}
            onValueChange={(value) => {
              if (isWorkspaceTheme(value)) onWorkspaceThemeChange(value);
            }}
            aria-label="Workspace theme"
            className="appearance-theme-options"
          >
            {WORKSPACE_THEMES.map((option) => {
              const palette = workspacePalette(option.id, theme);
              const preview = {
                '--preview-background': palette['bg-top'],
                '--preview-panel': palette.glass,
                '--preview-line': palette['glass-border'],
                '--preview-ink': palette['text-primary'],
                '--preview-muted': palette['recessed-border'],
                '--preview-accent': palette.accent,
                '--preview-radius': option.tokens['radius-panel'],
              } as CSSProperties;
              return (
                <label
                  className="appearance-theme-option"
                  key={option.id}
                  htmlFor={`workspace-${option.id}`}
                >
                  <div
                    className="appearance-theme-preview"
                    data-chrome={option.chrome}
                    style={preview}
                    aria-hidden="true"
                  >
                    <div className="appearance-preview-rail">
                      <b />
                      <span />
                      <span />
                      <span />
                    </div>
                    <div className="appearance-preview-body">
                      <span className="appearance-preview-title" />
                      <div className="appearance-preview-columns">
                        <div>
                          <span />
                          <span />
                          <i />
                          <b />
                        </div>
                        <div>
                          <span />
                          <i />
                          <i />
                          <span />
                        </div>
                      </div>
                    </div>
                  </div>
                  <div className="appearance-theme-label">
                    <span>{option.name}</span>
                    {option.id === DEFAULT_WORKSPACE_THEME && <small>Default</small>}
                    <RadioGroupItem
                      id={`workspace-${option.id}`}
                      value={option.id}
                      aria-label={option.name}
                      aria-describedby={`workspace-${option.id}-description`}
                    />
                  </div>
                  <p id={`workspace-${option.id}-description`}>{option.description}</p>
                </label>
              );
            })}
          </RadioGroup>
        </div>
      )}
      <p className="appearance-note">Applied across the app and saved on this device.</p>
    </div>
  );
}
