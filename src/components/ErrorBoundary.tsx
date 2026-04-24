import { Component, ErrorInfo, ReactNode } from 'react';

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export default class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('[ErrorBoundary]', error, errorInfo.componentStack);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="min-h-screen bg-bs-bg-primary flex items-center justify-center p-8">
          <div className="bg-bs-bg-tertiary border border-red-500/30 rounded-xl p-8 max-w-lg w-full">
            <h2 className="text-xl font-bold text-red-400 mb-4">应用发生错误</h2>
            <pre className="bg-bs-bg-primary rounded-lg p-4 text-sm text-bs-text-secondary overflow-auto max-h-48 mb-6">
              {this.state.error?.message}
            </pre>
            <button
              onClick={() => window.location.reload()}
              className="px-6 py-2.5 rounded-lg font-medium text-sm bg-bs-accent-primary text-white hover:bg-red-500 transition-all"
            >
              重新加载
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
