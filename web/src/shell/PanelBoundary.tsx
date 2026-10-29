import { Component, type ErrorInfo, type ReactNode } from 'react';

interface Props {
  name: string;
  children: ReactNode;
}

interface State {
  error: Error | null;
}

/**
 * Keeps a rendering bug in one panel from taking down the rest (docs/plan/08-ERRORS.md §6):
 * the editor and the other panels keep working, and this one offers a reload.
 */
export class PanelBoundary extends Component<Props, State> {
  override state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  override componentDidCatch(error: Error, info: ErrorInfo): void {
    console.error(`[Delta] ${this.props.name} panel crashed`, error, info.componentStack);
  }

  override render(): ReactNode {
    if (!this.state.error) return this.props.children;
    return (
      <div role="alert" className="m-4 rounded-md border border-reject/40 bg-reject/10 p-3 text-sm">
        <p className="font-medium text-text">The {this.props.name} panel hit a problem.</p>
        <p className="mt-1 font-mono text-xs text-muted">{this.state.error.message}</p>
        <button
          type="button"
          onClick={() => this.setState({ error: null })}
          className="mt-2 min-h-9 rounded-md border border-border px-3 text-text hover:bg-surface-2"
        >
          Reload panel
        </button>
      </div>
    );
  }
}
