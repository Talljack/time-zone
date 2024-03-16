import { getCurrent } from '@tauri-apps/api/webviewWindow'
import { UpdaterWindow } from './windows/UpdaterWindow'

const windowNameMap: Record<string, typeof UpdaterWindow> = {
  'updater': UpdaterWindow
}

export const App = () => {
  const currentWindowName = getCurrent()
  return <>
    { windowNameMap[currentWindowName.label]}
  </>
}