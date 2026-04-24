'use client';

import { useEffect, useState } from 'react';
import type { CountryCode } from 'libphonenumber-js/min';
import { COUNTRY_DIALS, parsePhone } from '@/lib/phone';
import { cn } from '@/lib/cn';

export type PhoneInputProps = {
  value: string;
  onChange: (value: string) => void;
  onValid: (valid: boolean, e164: string | null) => void;
  defaultCountry?: CountryCode;
  className?: string;
  id?: string;
};

export default function PhoneInput({
  value,
  onChange,
  onValid,
  defaultCountry = 'BR',
  className,
  id = 'phone-input',
}: PhoneInputProps) {
  const [country, setCountry] = useState<CountryCode>(defaultCountry);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!value) {
      setError(null);
      onValid(false, null);
      return;
    }
    const res = parsePhone(value, country);
    if (res.valid) {
      setError(null);
      onValid(true, res.e164);
    } else {
      setError(res.error);
      onValid(false, null);
    }
    // NB: onValid is from parent — we intentionally don't include it in deps
    // to avoid re-running on every render; the parent should memoize if needed.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [value, country]);

  return (
    <div className={cn('space-y-1.5', className)}>
      <label htmlFor={id} className="block text-sm font-medium text-wa-ink">
        Telefone (formato internacional)
      </label>
      <div className="flex gap-2">
        <select
          aria-label="País"
          value={country}
          onChange={(e) => setCountry(e.target.value as CountryCode)}
          className="w-40 rounded-md border border-wa-border bg-white px-2 py-2 text-sm shadow-sm focus:border-wa-primary focus:outline-none"
        >
          {COUNTRY_DIALS.map((c) => (
            <option key={c.code} value={c.code}>
              {c.label}
            </option>
          ))}
        </select>
        <input
          id={id}
          type="tel"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder="+55 11 99999-0000"
          aria-describedby={error ? `${id}-error` : undefined}
          aria-invalid={error ? true : undefined}
          className="flex-1 rounded-md border border-wa-border bg-white px-3 py-2 text-wa-ink shadow-sm focus:border-wa-primary focus:outline-none"
        />
      </div>
      {error && (
        <p id={`${id}-error`} className="text-sm text-red-600">
          {error}
        </p>
      )}
    </div>
  );
}
