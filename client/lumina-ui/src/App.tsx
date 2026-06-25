import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MonitorPlay, Copy, Settings, Search, Monitor, Plus, Command, X, ShieldCheck } from "lucide-react";
import { useScrambleText } from "./hooks/useScramble";

interface SavedMachine {
  id: string;
  name: string;
  is_online: boolean;
  last_connected: number;
}

function App() {
  const [deviceIdRaw, setDeviceIdRaw] = useState("Loading...");
  const deviceId = useScrambleText(deviceIdRaw, deviceIdRaw !== "Loading...");

  const [partnerId, setPartnerId] = useState("");
  const [isHoveringCopy, setIsHoveringCopy] = useState(false);
  
  // Auth Popup State
  const [showAuthPopup, setShowAuthPopup] = useState(false);
  const [pin, setPin] = useState("");
  const [saveMachine, setSaveMachine] = useState(true);
  const [isConnecting, setIsConnecting] = useState(false);
  
  // Saved Machines
  const [savedMachines, setSavedMachines] = useState<SavedMachine[]>([]);

  useEffect(() => {
    // Fetch local device ID from Rust backend
    invoke<string>("get_local_device_id")
      .then(setDeviceIdRaw)
      .catch(console.error);
      
    // Fetch saved machines
    loadSavedMachines();
  }, []);

  const loadSavedMachines = () => {
    invoke<SavedMachine[]>("get_saved_machines")
      .then(setSavedMachines)
      .catch(console.error);
  };

  const handleConnectClick = () => {
    if (partnerId.length > 5) {
      setShowAuthPopup(true);
    }
  };

  const handleAuthSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsConnecting(true);
    try {
      const response = await invoke<string>("connect_to_device", { 
        partnerId, 
        pin, 
        saveMachine 
      });
      alert(response);
      setShowAuthPopup(false);
      loadSavedMachines(); // Reload machines after successful save
    } catch (err) {
      alert(`Connection failed: ${err}`);
    } finally {
      setIsConnecting(false);
    }
  };

  return (
    <div className="min-h-screen bg-[#0E0F14] text-white flex flex-col p-6 overflow-hidden relative font-sans">
      
      {/* Auth Popup Overlay */}
      {showAuthPopup && (
        <div className="absolute inset-0 z-50 flex items-center justify-center bg-black/80" data-tauri-drag-region>
          <div className="glass-panel w-96 rounded-2xl p-6 relative border border-white/10">
            <button 
              onClick={() => setShowAuthPopup(false)}
              className="absolute top-4 right-4 text-gray-500 hover:text-white transition-colors"
            >
              <X size={20} />
            </button>
            
            <div className="flex items-center gap-3 mb-6">
              <div className="w-10 h-10 rounded-xl bg-indigo-500/20 flex items-center justify-center text-indigo-400">
                <ShieldCheck size={24} />
              </div>
              <div>
                <h3 className="font-medium text-lg">Авторизация</h3>
                <p className="text-xs text-gray-400">Доступ к {partnerId}</p>
              </div>
            </div>

            <form onSubmit={handleAuthSubmit}>
              <div className="mb-6">
                <label className="block text-xs font-medium text-gray-400 uppercase tracking-widest mb-2">Session PIN</label>
                <input 
                  type="text" 
                  value={pin}
                  onChange={(e) => setPin(e.target.value.toUpperCase())}
                  placeholder="XXXX-XXXX-XXXX"
                  className="w-full bg-black/50 border border-white/10 rounded-xl py-3 px-4 text-center text-xl font-mono tracking-widest text-indigo-300 placeholder-gray-700 outline-none focus:border-indigo-500/50 transition-all"
                  autoFocus
                />
              </div>

              <label className="flex items-center gap-3 mb-8 cursor-pointer group">
                <div className="relative flex items-center justify-center">
                  <input 
                    type="checkbox" 
                    checked={saveMachine}
                    onChange={(e) => setSaveMachine(e.target.checked)}
                    className="appearance-none w-5 h-5 border border-white/20 rounded bg-black/50 checked:bg-indigo-500 checked:border-indigo-500 transition-colors"
                  />
                  {saveMachine && <X size={14} className="absolute text-white pointer-events-none" style={{clipPath: "polygon(14% 44%, 0 65%, 50% 100%, 100% 16%, 80% 0%, 43% 62%)"}} />}
                </div>
                <span className="text-sm text-gray-300 group-hover:text-white transition-colors">Сохранить машину для Unattended Access</span>
              </label>

              <button 
                type="submit" 
                disabled={isConnecting || pin.length < 4}
                className="w-full py-3 rounded-xl bg-indigo-600 text-white font-medium hover:bg-indigo-500 transition-colors disabled:opacity-50 flex justify-center items-center gap-2"
              >
                {isConnecting ? (
                  <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                ) : (
                  "Подключиться"
                )}
              </button>
            </form>
          </div>
        </div>
      )}

      {/* Header */}
      <header className="flex justify-between items-center mb-8 relative z-10" data-tauri-drag-region>
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-indigo-600 flex items-center justify-center">
            <MonitorPlay size={20} className="text-white" />
          </div>
          <h1 className="text-2xl font-bold tracking-wider text-white">
            Lumina<span className="font-light text-indigo-400">Remote</span>
          </h1>
        </div>
        <button className="p-2 rounded-lg hover:bg-white/10 transition-colors">
          <Settings size={20} className="text-gray-400 hover:text-white transition-colors" />
        </button>
      </header>

      {/* Main Content */}
      <main className="flex-1 flex gap-6 relative z-10 h-full">
        
        {/* Left Column - Connection Dashboard */}
        <div className="flex flex-col gap-6 w-[45%]">
          
          {/* This Device Card */}
          <div className="glass-panel rounded-2xl p-8 relative overflow-hidden group">
            <div className="absolute top-0 left-0 w-full h-1 bg-indigo-500"></div>
            <h2 className="text-gray-400 text-sm font-medium uppercase tracking-widest mb-2">Это устройство</h2>
            <p className="text-gray-500 text-xs mb-6">Сообщите этот код партнеру для подключения</p>
            
            <div className="flex flex-col gap-3">
              <div 
                className="flex items-center justify-between bg-[#0E0F14] rounded-xl p-4 border border-[#2A2B32] cursor-pointer hover:border-indigo-500/50 transition-colors group/copy"
                onMouseEnter={() => setIsHoveringCopy(true)}
                onMouseLeave={() => setIsHoveringCopy(false)}
                onClick={() => navigator.clipboard.writeText(deviceIdRaw)}
                title="Копировать ID"
              >
                <div>
                  <div className="text-[10px] text-gray-500 mb-1 uppercase tracking-widest">Device ID</div>
                  <span className="text-2xl font-mono tracking-[0.2em] text-indigo-400">
                    {deviceId}
                  </span>
                </div>
                <div className={`p-2 rounded-lg transition-colors ${isHoveringCopy ? 'bg-indigo-500/20 text-indigo-400' : 'text-gray-600'}`}>
                  <Copy size={18} />
                </div>
              </div>
            </div>
            
            <div className="mt-6 flex items-center gap-2 text-xs text-emerald-400">
              <div className="w-2 h-2 rounded-full bg-emerald-400"></div>
              Готов к подключениям
            </div>
          </div>

          {/* Connect to Partner Card */}
          <div className="glass-panel rounded-2xl p-8 flex-1 flex flex-col">
            <h2 className="text-gray-400 text-sm font-medium uppercase tracking-widest mb-2">Подключиться</h2>
            <p className="text-gray-500 text-xs mb-6">Введите Device ID удаленного устройства</p>
            
            <div className="relative mb-6">
              <div className="absolute inset-y-0 left-4 flex items-center pointer-events-none">
                <Search size={18} className="text-gray-500" />
              </div>
              <input 
                type="text" 
                value={partnerId}
                onChange={(e) => setPartnerId(e.target.value.toUpperCase())}
                placeholder="XXXX-XXXX-XXXX"
                className="w-full bg-[#0E0F14] border border-[#2A2B32] rounded-xl py-4 pl-12 pr-4 text-xl font-mono tracking-widest text-white placeholder-gray-600 outline-none focus:border-indigo-500 transition-colors"
              />
            </div>

            <button 
              onClick={handleConnectClick}
              disabled={partnerId.length < 5}
              className="mt-auto w-full py-4 rounded-xl bg-indigo-600 text-white font-medium hover:bg-indigo-500 disabled:opacity-50 transition-colors flex items-center justify-center gap-2"
            >
              <Command size={18} />
              Подключиться
            </button>
          </div>
        </div>

        {/* Right Column - Saved Machines */}
        <div className="glass-panel rounded-2xl p-6 flex-1 flex flex-col">
          <div className="flex justify-between items-center mb-6">
            <h2 className="text-gray-300 text-sm font-medium uppercase tracking-widest">Мои машины</h2>
            <button className="p-1.5 rounded-lg bg-[#2A2B32] hover:bg-[#34353E] text-indigo-400 transition-colors">
              <Plus size={18} />
            </button>
          </div>
          
          {/* Grid of machines */}
          <div className="grid grid-cols-2 gap-4 overflow-y-auto pr-2">
            {savedMachines.length === 0 ? (
              <div className="col-span-2 text-center py-12 text-gray-500 text-sm">
                Нет сохраненных машин
              </div>
            ) : (
              savedMachines.map((machine) => (
                <div 
                  key={machine.id} 
                  className="group cursor-pointer"
                  onClick={() => {
                    setPartnerId(machine.id);
                    setShowAuthPopup(true);
                  }}
                >
                  <div className="aspect-video rounded-xl bg-[#0E0F14] border border-[#2A2B32] overflow-hidden relative mb-3 group-hover:border-indigo-500/50 transition-colors flex items-center justify-center">
                    <Monitor size={48} className="text-[#2A2B32] group-hover:text-indigo-500/50 transition-colors" />
                    <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity flex items-end p-3">
                      <span className="text-xs font-medium text-white bg-indigo-500/80 px-2 py-1 rounded">Подключиться</span>
                    </div>
                  </div>
                  <h3 className="text-sm font-medium text-gray-200">{machine.name}</h3>
                  <p className="text-xs text-emerald-400 mt-1 flex items-center gap-1">
                    <span className="w-1.5 h-1.5 rounded-full bg-emerald-400"></span> Saved
                  </p>
                </div>
              ))
            )}
          </div>
          
        </div>
      </main>
    </div>
  );
}

export default App;
