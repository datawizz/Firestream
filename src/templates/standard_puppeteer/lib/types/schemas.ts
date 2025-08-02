// Generated TypeScript types

export interface DataSchema {
  id: string;
  name: string;
  description?: string;
  fields: SchemaField[];
}

export interface SchemaField {
  name: string;
  type: 'string' | 'number' | 'boolean' | 'array' | 'object' | 'date';
  required?: boolean;
  description?: string;
  default?: any;
  format?: string;
  items?: SchemaField;
  properties?: Record<string, SchemaField>;
}