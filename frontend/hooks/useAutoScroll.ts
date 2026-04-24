'use client';

import { useEffect, useRef } from 'react';

// Returns a ref to attach to a scrollable container; scrolls to bottom when
// `dep` changes (e.g. message list length).
export function useAutoScroll<T>(dep: T) {
  const ref = useRef<HTMLDivElement | null>(null);
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    el.scrollTop = el.scrollHeight;
  }, [dep]);
  return ref;
}
