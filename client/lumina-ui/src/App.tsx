import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
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
  const [isConnected, setIsConnected] = useState(false);
  
  const [savedMachines, setSavedMachines] = useState<SavedMachine[]>([]);
  const [lanMachines, setLanMachines] = useState<SavedMachine[]>([]);
  
  // Host PIN
  const [hostPinRaw, setHostPinRaw] = useState("••••-••••-••••");
  const hostPin = useScrambleText(hostPinRaw, hostPinRaw !== "••••-••••-••••");

  // Video Stream State
  const [videoFrame, setVideoFrame] = useState<string | null>(null);

  // Incoming Connection State
  const [incomingConnection, setIncomingConnection] = useState<string | null>(null);

  useEffect(() => {
    invoke<string>("get_local_device_id")
      .then(setDeviceIdRaw)
      .catch(console.error);
      
    // Generate host pin immediately
    invoke<string>("generate_session_pin")
      .then(setHostPinRaw)
      .catch(console.error);
      
    loadSavedMachines();
    
    // Poll LAN devices every 5 seconds
    const lanInterval = setInterval(() => {
      invoke<SavedMachine[]>("get_local_network_devices")
        .then(setLanMachines)
        .catch(console.error);
    }, 5000);
    
    // Initial fetch
    invoke<SavedMachine[]>("get_local_network_devices")
      .then(setLanMachines)
      .catch(console.error);

    // Listen for incoming LAN connections
    const unlistenConnection = listen<string>("incoming-connection", (event) => {
      console.log("Incoming connection from:", event.payload);
      setIncomingConnection(event.payload);
    });

    // Listen for video frames
    const unlistenVideo = listen<string>("video-frame", (event) => {
      setVideoFrame(`data:image/jpeg;base64,${event.payload}`);
    });

    return () => {
      clearInterval(lanInterval);
      unlistenConnection.then(f => f());
      unlistenVideo.then(f => f());
    };
  }, []);

  const loadSavedMachines = () => {
    invoke<SavedMachine[]>("get_saved_machines")
      .then(setSavedMachines)
      .catch(console.error);
  };

  const allMachines = [...lanMachines, ...savedMachines.filter(s => !lanMachines.find(l => l.id === s.id))];

  const handleConnectClick = () => {
    if (partnerId.length > 5) {
      setShowAuthPopup(true);
    }
  };

  const handleAuthSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsConnecting(true);
    try {
      setIsConnecting(true);
      invoke<string>("connect_to_device", { partnerId, pin, saveMachine })
        .then(() => {
          setIsConnecting(false);
          setShowAuthPopup(false);
          setIsConnected(true);
        })
        .catch((err) => {
          setIsConnecting(false);
          alert(err);
        });
    } catch (err) {
      alert(`Connection failed: ${err}`);
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  const formatDeviceId = (val: string) => {
    const cleaned = val.toUpperCase().replace(/[^A-Z0-9]/g, '');
    if (cleaned.length <= 3) return cleaned;
    if (cleaned.length <= 7) return `${cleaned.slice(0, 3)}-${cleaned.slice(3)}`;
    return `${cleaned.slice(0, 3)}-${cleaned.slice(3, 7)}-${cleaned.slice(7, 11)}`;
  };

  const formatPin = (val: string) => {
    const cleaned = val.toUpperCase().replace(/[^A-Z0-9]/g, '');
    if (cleaned.length <= 4) return cleaned;
    if (cleaned.length <= 8) return `${cleaned.slice(0, 4)}-${cleaned.slice(4)}`;
    return `${cleaned.slice(0, 4)}-${cleaned.slice(4, 8)}-${cleaned.slice(8, 12)}`;
  };

  if (isConnected) {
    return (
      <div className="min-h-screen bg-black text-white flex flex-col relative overflow-hidden" data-tauri-drag-region>
        <div className="absolute top-0 left-0 w-full h-12 bg-gradient-to-b from-black/80 to-transparent z-10 pointer-events-none" />
        {videoFrame ? (
          <img 
            src={videoFrame} 
            alt="Remote Desktop Stream" 
            className="w-full h-full object-contain pointer-events-none"
          />
        ) : (
          <div className="flex-1 flex flex-col items-center justify-center">
            <div className="w-16 h-16 border-4 border-indigo-500 border-t-transparent rounded-full animate-spin mb-4" />
            <p className="text-gray-400">Ожидание видеопотока...</p>
          </div>
        )}
        <button 
          onClick={() => setIsConnected(false)}
          className="absolute top-4 right-4 z-20 px-4 py-2 bg-red-500/80 hover:bg-red-500 text-white rounded-lg transition-colors backdrop-blur-md"
        >
          Отключиться
        </button>
      </div>
    );
  }

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
                  onChange={(e) => setPin(formatPin(e.target.value))}
                  placeholder="XXXX-XXXX-XXXX"
                  maxLength={14}
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

      {/* Incoming Connection Popup Overlay */}
      {incomingConnection && (
        <div className="absolute inset-0 z-50 flex items-center justify-center bg-black/80" data-tauri-drag-region>
          <div className="glass-panel w-96 rounded-2xl p-6 relative border border-emerald-500/30 shadow-[0_0_50px_-12px_rgba(16,185,129,0.3)]">
            <div className="flex flex-col items-center text-center gap-4">
              <div className="w-16 h-16 rounded-full bg-emerald-500/20 flex items-center justify-center text-emerald-400 mb-2 relative">
                <ShieldCheck size={32} />
                <div className="absolute inset-0 border-2 border-emerald-500 rounded-full animate-ping opacity-20"></div>
              </div>
              
              <div>
                <h3 className="font-bold text-xl text-white mb-1">Входящее подключение</h3>
                <p className="text-sm text-gray-400">Устройство <span className="font-mono text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded">{incomingConnection}</span> хочет получить доступ к вашему рабочему столу.</p>
              </div>

              <div className="flex gap-3 w-full mt-4">
                <button 
                  onClick={() => setIncomingConnection(null)}
                  className="flex-1 py-3 rounded-xl bg-red-500/10 text-red-400 font-medium hover:bg-red-500/20 transition-colors border border-red-500/20"
                >
                  Отклонить
                </button>
                <button 
                  onClick={() => {
                    // For alpha, the Rust backend automatically replies ACCEPTED,
                    // but we dismiss the popup here.
                    setIncomingConnection(null);
                  }}
                  className="flex-1 py-3 rounded-xl bg-emerald-600 text-white font-medium hover:bg-emerald-500 transition-colors shadow-lg shadow-emerald-600/20"
                >
                  Разрешить
                </button>
              </div>
            </div>
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

              {/* Host PIN Display */}
              <div className="mt-4">
                <label className="text-[10px] text-gray-500 mb-1 uppercase tracking-widest block">Session PIN (Пароль)</label>
                <div className="flex items-center gap-2">
                  <div className="bg-[#0E0F14] rounded-xl p-3 border border-[#2A2B32] flex-1 text-center font-mono text-xl tracking-widest text-emerald-400">
                    {hostPin}
                  </div>
                  <button 
                    onClick={() => copyToClipboard(hostPinRaw)}
                    className="p-3 bg-[#0E0F14] border border-[#2A2B32] hover:border-indigo-500/50 hover:text-indigo-400 rounded-xl transition-colors text-gray-400"
                    title="Копировать пароль"
                  >
                    <Copy size={20} />
                  </button>
                  <button 
                    onClick={() => invoke<string>("generate_session_pin").then(setHostPinRaw)}
                    className="p-3 bg-[#0E0F14] border border-[#2A2B32] hover:border-indigo-500/50 hover:text-indigo-400 rounded-xl transition-colors text-gray-400"
                    title="Сгенерировать новый пароль"
                  >
                    <Settings size={20} />
                  </button>
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
                onChange={(e) => setPartnerId(formatDeviceId(e.target.value))}
                placeholder="LMN-XXXX-XXXX"
                maxLength={13}
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
          
          <div className="grid grid-cols-2 gap-4 overflow-y-auto pr-2">
            {allMachines.length === 0 ? (
              <div className="col-span-2 text-center py-12 text-gray-500 text-sm">
                Нет сохраненных машин
              </div>
            ) : (
              allMachines.map((machine) => (
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
                  <p className="text-xs mt-1 flex items-center gap-1">
                    <span className={`w-1.5 h-1.5 rounded-full ${machine.name.includes('(LAN)') ? 'bg-blue-400' : 'bg-emerald-400'}`}></span> 
                    {machine.name.includes('(LAN)') ? <span className="text-blue-400">В локальной сети</span> : <span className="text-emerald-400">Сохранено</span>}
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
