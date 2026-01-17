// Data Schemas
// Auto-generated from workflow definition

import { z } from 'zod';

export const ProductSchema = z.object({
  id: z.string(),
  name: z.string(),
  price: z.number().optional(),
  inStock: z.boolean().optional(),
});

export type Product = z.infer<typeof ProductSchema>;

