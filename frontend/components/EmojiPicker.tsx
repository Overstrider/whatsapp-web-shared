'use client';

import dynamic from 'next/dynamic';
import { useEffect, useRef } from 'react';
import LoadingDots from './LoadingDots';

// Lazy load — emoji-picker-react ships a big bundle; only cost it when used.
const Picker = dynamic(() => import('emoji-picker-react'), {
  ssr: false,
  loading: () => (
    <div className="flex h-72 w-72 items-center justify-center rounded-md bg-white shadow">
      <LoadingDots />
    </div>
  ),
});

export type EmojiPickerProps = {
  onSelect: (emoji: string) => void;
  onClose: () => void;
};

export default function EmojiPicker({ onSelect, onClose }: EmojiPickerProps) {
  const wrapRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    function onDocClick(e: MouseEvent) {
      if (!wrapRef.current) return;
      if (!wrapRef.current.contains(e.target as Node)) onClose();
    }
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') onClose();
    }
    document.addEventListener('mousedown', onDocClick);
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('mousedown', onDocClick);
      document.removeEventListener('keydown', onKey);
    };
  }, [onClose]);

  return (
    <div
      ref={wrapRef}
      role="dialog"
      aria-label="Seletor de emoji"
      className="absolute bottom-14 left-2 z-40 overflow-hidden rounded-md shadow-lg"
    >
      <Picker
        onEmojiClick={(data) => onSelect(data.emoji)}
        autoFocusSearch
        width={320}
        height={360}
      />
    </div>
  );
}
