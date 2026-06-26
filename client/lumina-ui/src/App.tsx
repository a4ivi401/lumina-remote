import { useState, useEffect, useRef } from "react";
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
  


  // Video Stream State
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const decoderRef = useRef<VideoDecoder | null>(null);
  const [hasVideo, setHasVideo] = useState(false);

  // Incoming Connection State
  const [incomingConnection, setIncomingConnection] = useState<{partner: string, pin: string} | null>(null);

  // Signal Server State
  const [signalServerConnected, setSignalServerConnected] = useState<boolean | null>(null);

  useEffect(() => {
    invoke<string>("get_local_device_id")
      .then(setDeviceIdRaw)
      .catch(console.error);
      

      
    loadSavedMachines();
    
    // Poll LAN devices and Signal Server every 5 seconds
    const interval = setInterval(() => {
      invoke<SavedMachine[]>("get_local_network_devices")
        .then(setLanMachines)
        .catch(console.error);
        
      invoke<boolean>("check_signal_server")
        .then(setSignalServerConnected)
        .catch(() => setSignalServerConnected(false));
    }, 5000);
    
    // Initial fetch
    invoke<SavedMachine[]>("get_local_network_devices")
      .then(setLanMachines)
      .catch(console.error);
      
    invoke<boolean>("check_signal_server")
      .then(setSignalServerConnected)
      .catch(() => setSignalServerConnected(false));

    // Listen for incoming connections
    const unlistenConnection = listen<{ partner: string, pin: string }>("incoming-connection", (event) => {
      console.log("Incoming connection from:", event.payload.partner);
      setIncomingConnection(event.payload);
    });

    // Listen for video frames
    const unlistenVideo = listen<{ data: string, is_keyframe: boolean, timestamp_us: number }>("video-frame", (event) => {
      if (!decoderRef.current && canvasRef.current) {
        const ctx = canvasRef.current.getContext('2d');
        if (!ctx) return;
        
        const decoder = new VideoDecoder({
          output(frame) {
            setHasVideo(true);
            if (canvasRef.current) {
              if (canvasRef.current.width !== frame.displayWidth || canvasRef.current.height !== frame.displayHeight) {
                canvasRef.current.width = frame.displayWidth;
                canvasRef.current.height = frame.displayHeight;
              }
              ctx.drawImage(frame, 0, 0, canvasRef.current.width, canvasRef.current.height);
            }
            frame.close();
          },
          error(e) { console.error("VideoDecoder error:", e); }
        });
        
        decoder.configure({ codec: 'avc1.42E01E' });
        decoderRef.current = decoder;
      }
      
      if (decoderRef.current && decoderRef.current.state === "configured") {
        try {
          const binaryString = atob(event.payload.data);
          const bytes = new Uint8Array(binaryString.length);
          for (let i = 0; i < binaryString.length; i++) {
            bytes[i] = binaryString.charCodeAt(i);
          }
          
          const chunk = new EncodedVideoChunk({
            type: event.payload.is_keyframe ? 'key' : 'delta',
            timestamp: event.payload.timestamp_us,
            data: bytes
          });
          
          decoderRef.current.decode(chunk);
        } catch (e) {
          console.error("Failed to decode chunk", e);
        }
      }
    });

    return () => {
      clearInterval(interval);
      unlistenConnection.then(f => f());
      unlistenVideo.then(f => f());
      if (decoderRef.current && decoderRef.current.state !== "closed") {
        decoderRef.current.close();
        decoderRef.current = null;
      }
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
      invoke("trigger_connection_request", { partnerId }).catch(console.error);
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

  const handleMouseMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!isConnected) return;
    const rect = e.currentTarget.getBoundingClientRect();
    // Simple scaling mapping (assuming remote and local resolutions align for now)
    const x = Math.round(e.clientX - rect.left);
    const y = Math.round(e.clientY - rect.top);
    invoke("send_input", { event: JSON.stringify({ MouseMove: { x, y } }) });
  };

  const handleMouseClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!isConnected) return;
    let button = "left";
    if (e.button === 2) button = "right";
    if (e.button === 1) button = "middle";
    invoke("send_input", { event: JSON.stringify({ MouseClick: { button } }) });
  };

  useEffect(() => {
    if (!isConnected) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      let key = e.key.toLowerCase();
      if (key === " ") key = "space";
      invoke("send_input", { event: JSON.stringify({ KeyDown: { key } }) });
      e.preventDefault();
    };
    const handleKeyUp = (e: KeyboardEvent) => {
      let key = e.key.toLowerCase();
      if (key === " ") key = "space";
      invoke("send_input", { event: JSON.stringify({ KeyUp: { key } }) });
      e.preventDefault();
    };
    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
    };
  }, [isConnected]);

  if (isConnected) {
    return (
      <div className="min-h-screen bg-black text-white flex flex-col relative overflow-hidden" data-tauri-drag-region>
        <div className="absolute top-0 left-0 w-full h-12 bg-gradient-to-b from-black/80 to-transparent z-10 pointer-events-none" />
        <canvas 
          ref={canvasRef}
          className="w-full h-full object-contain absolute inset-0 z-0"
          onMouseMove={handleMouseMove}
          onClick={handleMouseClick}
          onContextMenu={(e) => { e.preventDefault(); handleMouseClick(e); }}
        />
        {!hasVideo && (
          <div className="absolute inset-0 flex flex-col items-center justify-center z-10 pointer-events-none bg-black">
            <div className="w-16 h-16 border-4 border-indigo-500 border-t-transparent rounded-full animate-spin mb-4" />
            <p className="text-gray-400">Получение защищенного H.264 видеопотока...</p>
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
                <p className="text-sm text-gray-400 mb-4">Устройство <span className="font-mono text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded">{incomingConnection.partner}</span> хочет получить доступ к вашему рабочему столу.</p>
                
                <div className="bg-black/50 border border-emerald-500/30 rounded-xl p-4 mt-2">
                  <p className="text-xs text-gray-400 uppercase tracking-widest mb-2">Сообщите этот PIN-код:</p>
                  <div className="font-mono text-2xl tracking-[0.2em] text-emerald-400">{incomingConnection.pin}</div>
                </div>
              </div>

              <div className="flex gap-3 w-full mt-4">
                <button 
                  onClick={() => {
                    invoke("respond_to_connection", { accept: false });
                    setIncomingConnection(null);
                  }}
                  className="flex-1 py-3 rounded-xl bg-red-500/10 text-red-400 font-medium hover:bg-red-500/20 transition-colors border border-red-500/20"
                >
                  Отклонить
                </button>
                <button 
                  onClick={() => {
                    invoke("respond_to_connection", { accept: true });
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

              {/* Host PIN is now dynamically generated on connection attempt */}
              <div className="mt-4 bg-indigo-500/10 border border-indigo-500/20 rounded-xl p-4">
                <p className="text-xs text-indigo-300">
                  PIN-код генерируется автоматически при входящем запросе на подключение.
                </p>
              </div>
            </div>
            
            <div className="mt-6 flex flex-col gap-2 text-xs">
              <div className="flex items-center gap-2 text-emerald-400">
                <div className="w-2 h-2 rounded-full bg-emerald-400"></div>
                Служба локальной сети работает (LAN)
              </div>
              <div className={`flex items-center gap-2 ${signalServerConnected === true ? 'text-emerald-400' : signalServerConnected === false ? 'text-red-400' : 'text-gray-400'}`}>
                <div className={`w-2 h-2 rounded-full ${signalServerConnected === true ? 'bg-emerald-400' : signalServerConnected === false ? 'bg-red-400' : 'bg-gray-400 animate-pulse'}`}></div>
                {signalServerConnected === true 
                  ? 'Связь с сигнальным сервером установлена (WAN)' 
                  : signalServerConnected === false 
                    ? 'Нет связи с сигнальным сервером' 
                    : 'Проверка связи с сервером...'}
              </div>
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
