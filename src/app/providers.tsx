import type { PropsWithChildren } from 'react';
import { LocalChallengeProvider } from '../services/local/LocalChallengeProvider';
import { LocalScoreProvider } from '../services/local/LocalScoreProvider';
import {
  ServicesContext,
  type AppServices,
} from './servicesContext';

const localServices: AppServices = {
  challengeProvider: new LocalChallengeProvider(),
  scoreProvider: new LocalScoreProvider(),
};

interface AppProvidersProps extends PropsWithChildren {
  services?: AppServices;
}

export function AppProviders({
  children,
  services = localServices,
}: AppProvidersProps) {
  return (
    <ServicesContext.Provider value={services}>
      {children}
    </ServicesContext.Provider>
  );
}
