import { Dashboard } from "@/components/dashboard";
import { AppProvider } from "@/contexts/AppContext";
import { ThemeProvider } from "@/lib/theme";

export function App() {
  return (
    <ThemeProvider>
      <AppProvider>
        <Dashboard />
      </AppProvider>
    </ThemeProvider>
  );
}

export default App;
