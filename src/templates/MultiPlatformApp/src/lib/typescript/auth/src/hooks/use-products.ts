/**
 * useProducts hook
 *
 * React hook for fetching available products and pricing plans
 *
 * @packageDocumentation
 */

import { useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useSubscriptionStore } from '../stores/subscription-store';
import { useSubscriptionService } from '../context/auth-provider';
import type { ProductWithPrices } from '../types/subscription';

/**
 * Products hook
 *
 * Fetches all available products with their pricing information.
 * Automatically syncs with the subscription store.
 *
 * @example
 * ```tsx
 * function PricingPage() {
 *   const { products, isLoading, error } = useProducts();
 *
 *   if (isLoading) return <p>Loading plans...</p>;
 *   if (error) return <p>Error loading plans</p>;
 *
 *   return (
 *     <div>
 *       {products.map(product => (
 *         <div key={product.id}>
 *           <h3>{product.name}</h3>
 *           <p>{product.description}</p>
 *           {product.prices.map(price => (
 *             <div key={price.id}>
 *               <p>{price.unit_amount / 100} / {price.interval}</p>
 *               <button onClick={() => handleSubscribe(price.id)}>
 *                 Subscribe
 *               </button>
 *             </div>
 *           ))}
 *         </div>
 *       ))}
 *     </div>
 *   );
 * }
 * ```
 */
export function useProducts() {
  const subscriptionService = useSubscriptionService();
  const { products: storedProducts, setProducts } = useSubscriptionStore();

  // Fetch all products with prices
  const {
    data: products,
    isLoading,
    error,
    refetch,
  } = useQuery({
    queryKey: ['products'],
    queryFn: async () => {
      const result = await subscriptionService.getProducts();
      return result.data ?? [];
    },
    staleTime: 1000 * 60 * 30, // 30 minutes - products don't change often
    retry: 1,
  });

  // Sync products to store
  useEffect(() => {
    if (products !== undefined) {
      setProducts(products);
    }
  }, [products, setProducts]);

  /**
   * Get a specific product by ID
   */
  const getProduct = (productId: string): ProductWithPrices | undefined => {
    const allProducts = products ?? storedProducts;
    return allProducts.find((p) => p.id === productId);
  };

  /**
   * Get active (available for purchase) products
   */
  const getActiveProducts = (): ProductWithPrices[] => {
    const allProducts = products ?? storedProducts;
    return allProducts.filter((p) => p.active);
  };

  /**
   * Get products by metadata key-value pair
   */
  const getProductsByMetadata = (key: string, value: string): ProductWithPrices[] => {
    const allProducts = products ?? storedProducts;
    return allProducts.filter(
      (p) =>
        p.metadata &&
        typeof p.metadata === 'object' &&
        !Array.isArray(p.metadata) &&
        p.metadata[key] === value
    );
  };

  return {
    // State
    products: (products ?? storedProducts) as ProductWithPrices[],
    isLoading,
    error: error as Error | null,

    // Methods
    refetch,
    getProduct,
    getActiveProducts,
    getProductsByMetadata,
  };
}
