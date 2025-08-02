// Generate 100 realistic mock products
const categories = ['Electronics', 'Clothing', 'Home & Garden', 'Sports', 'Books'];
const adjectives = ['Premium', 'Deluxe', 'Essential', 'Professional', 'Classic', 'Modern', 'Vintage', 'Smart', 'Eco-Friendly', 'Luxury'];
const productTypes = {
  Electronics: ['Smartphone', 'Laptop', 'Headphones', 'Tablet', 'Camera', 'Smartwatch', 'Speaker', 'Monitor'],
  Clothing: ['T-Shirt', 'Jeans', 'Jacket', 'Dress', 'Shoes', 'Sweater', 'Hat', 'Socks'],
  'Home & Garden': ['Chair', 'Table', 'Lamp', 'Rug', 'Plant Pot', 'Cushion', 'Vase', 'Mirror'],
  Sports: ['Running Shoes', 'Yoga Mat', 'Dumbbell Set', 'Tennis Racket', 'Basketball', 'Cycling Helmet', 'Water Bottle'],
  Books: ['Novel', 'Cookbook', 'Travel Guide', 'Biography', 'Science Fiction', 'Self-Help', 'History', 'Art Book']
};

const generateProducts = () => {
  const products = [];
  
  for (let i = 1; i <= 100; i++) {
    const category = categories[Math.floor(Math.random() * categories.length)];
    const productType = productTypes[category][Math.floor(Math.random() * productTypes[category].length)];
    const adjective = adjectives[Math.floor(Math.random() * adjectives.length)];
    
    // Price between $9.99 and $999.99
    const price = (Math.random() * 990 + 9.99).toFixed(2);
    
    // 80% in stock, 20% out of stock
    const inStock = Math.random() > 0.2;
    
    products.push({
      id: `PROD-${String(i).padStart(4, '0')}`,
      name: `${adjective} ${productType}`,
      price: parseFloat(price),
      category,
      inStock,
      description: `High-quality ${productType.toLowerCase()} perfect for your ${category.toLowerCase()} needs.`,
      rating: (Math.random() * 2 + 3).toFixed(1), // Rating between 3.0 and 5.0
      reviews: Math.floor(Math.random() * 500)
    });
  }
  
  return products;
};

const mockProducts = generateProducts();

// ES6 export for React components
export { mockProducts };

// Also support CommonJS for Node.js server
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { mockProducts };
}