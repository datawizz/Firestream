// Effect Registry
// Manages effect instances and provides type-safe access

import { Effect } from '../effects';
import { logger } from '../utils/logger';

export class EffectRegistry {
  private effects: Map<string, Effect<any, any, any>> = new Map();
  private dependencies: Map<string, string[]> = new Map();
  
  /**
   * Register an effect in the registry
   */
  register(effect: Effect<any, any, any>): void {
    if (this.effects.has(effect.id)) {
      throw new Error(`Effect with id "${effect.id}" is already registered`);
    }
    
    this.effects.set(effect.id, effect);
    this.dependencies.set(effect.id, effect.dependencies);
    
    logger.debug(`Registered effect: ${effect.id} (${effect.type})`);
  }
  
  /**
   * Get an effect by ID
   */
  get(id: string): Effect<any, any, any> | undefined {
    return this.effects.get(id);
  }
  
  /**
   * Get all effect IDs
   */
  getIds(): string[] {
    return Array.from(this.effects.keys());
  }
  
  /**
   * Get all effects
   */
  getAll(): Map<string, Effect<any, any, any>> {
    return new Map(this.effects);
  }
  
  /**
   * Validate all dependencies exist
   */
  validateDependencies(): void {
    for (const [effectId, deps] of this.dependencies.entries()) {
      for (const dep of deps) {
        if (!this.effects.has(dep)) {
          throw new Error(
            `Effect "${effectId}" depends on non-existent effect "${dep}"`
          );
        }
      }
    }
  }
  
  /**
   * Get execution order using topological sort
   */
  getExecutionOrder(): string[] {
    const inDegree = new Map<string, number>();
    const graph = new Map<string, string[]>();
    
    // Initialize in-degree counts and reverse graph
    for (const [id, deps] of this.dependencies.entries()) {
      inDegree.set(id, deps.length);
      
      // Build reverse graph for topological sort
      for (const dep of deps) {
        if (!graph.has(dep)) {
          graph.set(dep, []);
        }
        graph.get(dep)!.push(id);
      }
    }
    
    // Find all nodes with no dependencies
    const queue: string[] = [];
    for (const [id, degree] of inDegree.entries()) {
      if (degree === 0) {
        queue.push(id);
      }
    }
    
    // Process queue
    const order: string[] = [];
    while (queue.length > 0) {
      const current = queue.shift()!;
      order.push(current);
      
      // Update dependent nodes
      const dependents = graph.get(current) || [];
      for (const dependent of dependents) {
        const newDegree = inDegree.get(dependent)! - 1;
        inDegree.set(dependent, newDegree);
        
        if (newDegree === 0) {
          queue.push(dependent);
        }
      }
    }
    
    // Check for circular dependencies
    if (order.length !== this.effects.size) {
      const unprocessed = Array.from(this.effects.keys())
        .filter(id => !order.includes(id));
      throw new Error(
        `Circular dependency detected. Unprocessed effects: ${unprocessed.join(', ')}`
      );
    }
    
    return order;
  }
  
  /**
   * Get direct dependencies of an effect
   */
  getDependencies(effectId: string): string[] {
    return this.dependencies.get(effectId) || [];
  }
  
  /**
   * Get all effects that depend on a given effect
   */
  getDependents(effectId: string): string[] {
    const dependents: string[] = [];
    
    for (const [id, deps] of this.dependencies.entries()) {
      if (deps.includes(effectId)) {
        dependents.push(id);
      }
    }
    
    return dependents;
  }
  
  /**
   * Clear the registry
   */
  clear(): void {
    this.effects.clear();
    this.dependencies.clear();
  }
  
  /**
   * Get registry statistics
   */
  getStats(): {
    totalEffects: number;
    effectsByType: Record<string, number>;
    averageDependencies: number;
    maxDependencies: number;
  } {
    const effectsByType: Record<string, number> = {};
    let totalDependencies = 0;
    let maxDependencies = 0;
    
    for (const effect of this.effects.values()) {
      // Count by type
      effectsByType[effect.type] = (effectsByType[effect.type] || 0) + 1;
      
      // Count dependencies
      const depCount = effect.dependencies.length;
      totalDependencies += depCount;
      maxDependencies = Math.max(maxDependencies, depCount);
    }
    
    return {
      totalEffects: this.effects.size,
      effectsByType,
      averageDependencies: this.effects.size > 0 
        ? totalDependencies / this.effects.size 
        : 0,
      maxDependencies
    };
  }
}