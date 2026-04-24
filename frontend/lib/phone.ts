import { parsePhoneNumberFromString, type CountryCode } from 'libphonenumber-js/min';

export type PhoneParseResult =
  | { valid: true; e164: string; country: CountryCode }
  | { valid: false; error: string };

export function parsePhone(input: string, defaultCountry: CountryCode = 'BR'): PhoneParseResult {
  const trimmed = input.trim();
  if (!trimmed) return { valid: false, error: 'Informe um número.' };
  try {
    const parsed = parsePhoneNumberFromString(trimmed, defaultCountry);
    if (!parsed || !parsed.isValid()) {
      return { valid: false, error: 'Número inválido.' };
    }
    return {
      valid: true,
      e164: parsed.number,
      country: (parsed.country ?? defaultCountry) as CountryCode,
    };
  } catch {
    return { valid: false, error: 'Número inválido.' };
  }
}

export const COUNTRY_DIALS: Array<{ code: CountryCode; dial: string; label: string }> = [
  { code: 'BR', dial: '+55', label: 'Brasil (+55)' },
  { code: 'US', dial: '+1', label: 'USA (+1)' },
  { code: 'PT', dial: '+351', label: 'Portugal (+351)' },
  { code: 'AR', dial: '+54', label: 'Argentina (+54)' },
  { code: 'ES', dial: '+34', label: 'España (+34)' },
  { code: 'GB', dial: '+44', label: 'UK (+44)' },
];
