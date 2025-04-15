import { analytics } from '@vercel/analytics';
import React from 'react';

// Use React but not analytics
export function Component() {
  return React.createElement('div', null, 'No analytics used');
}
