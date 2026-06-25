import { useState, useEffect } from 'react';

const ALPHABET = '23456789ABCDEFGHJKLMNPQRSTUVWXYZ';

export function useScrambleText(text: string, isActive: boolean = true) {
  const [displayText, setDisplayText] = useState(text);

  useEffect(() => {
    if (!isActive || !text || text === 'Loading...') {
      setDisplayText(text);
      return;
    }

    let iteration = 0;
    const maxIterations = 20; // How many times it scrambles before settling
    
    const interval = setInterval(() => {
      setDisplayText((_prev) => {
        return text
          .split('')
          .map((char, index) => {
            // Keep dashes intact
            if (char === '-') return '-';
            
            // If the iteration has passed this character's random threshold, show correct char
            if (index < iteration / (maxIterations / text.length)) {
              return text[index];
            }
            
            // Otherwise, show random char
            return ALPHABET[Math.floor(Math.random() * ALPHABET.length)];
          })
          .join('');
      });

      iteration += 1;
      
      if (iteration >= maxIterations) {
        clearInterval(interval);
        setDisplayText(text);
      }
    }, 40); // 40ms per frame

    return () => clearInterval(interval);
  }, [text, isActive]);

  return displayText;
}
