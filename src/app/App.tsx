import { AppProviders } from './providers';
import { WorkbenchBootstrap } from './WorkbenchBootstrap';

export function App() {
  return (
    <AppProviders>
      <WorkbenchBootstrap />
    </AppProviders>
  );
}
