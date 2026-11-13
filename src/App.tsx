import Popup from "./windows/Popup";
import Settings from "./windows/Settings";
import { I18nProvider } from "./i18n";

interface AppProps {
  windowLabel: string;
}

export default function App({ windowLabel }: AppProps) {
  return (
    <I18nProvider>
      {windowLabel === "settings" ? <Settings /> : <Popup />}
    </I18nProvider>
  );
}
