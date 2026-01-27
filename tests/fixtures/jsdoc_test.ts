// Test file for JSDoc and comment extraction

/**
 * Represents a product in the store.
 * @property {number} id - Unique identifier
 * @property {string} name - Product name
 * @property {number} price - Price in cents
 */
interface Product {
  id: number;
  name: string;
  price: number;
}

/**
 * ProductService handles all product-related operations.
 *
 * @example
 * const service = new ProductService();
 * service.addProduct({ id: 1, name: "Widget", price: 999 });
 */
class ProductService {
  private products: Product[] = [];

  /**
   * Adds a new product to the store.
   * @param product - The product to add
   * @returns The added product with generated ID
   */
  addProduct(product: Product): Product {
    this.products.push(product);
    return product;
  }

  // TODO: Implement search functionality
  // FIXME: This method has a bug with empty arrays
  findProduct(id: number): Product | undefined {
    return this.products.find(p => p.id === id);
  }
}

// Helper function for formatting
// Returns formatted price string
function formatPrice(price: number): string {
  return `$${(price / 100).toFixed(2)}`;
}

/*
 * Multi-line block comment
 * Contains a TODO item
 * TODO: Add currency support
 */
const DEFAULT_CURRENCY = "USD";

/* Single line block comment with FIXME: needs update */
const VERSION = "1.0.0";

/**
 * Creates a new product with default values.
 * @param name - Product name
 * @returns A new Product object
 */
const createProduct = (name: string): Product => {
  return {
    id: Date.now(),
    name,
    price: 0,
  };
};

export { Product, ProductService, formatPrice, createProduct };
