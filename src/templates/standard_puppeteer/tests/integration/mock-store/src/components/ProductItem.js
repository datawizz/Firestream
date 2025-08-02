import React from 'react';

const ProductItem = ({ product }) => {
  return (
    <div className="product-item" data-product-id={product.id}>
      <h3 className="product-name">{product.name}</h3>
      <div className="product-price">${product.price.toFixed(2)}</div>
      <div className={`availability ${product.inStock ? 'in-stock' : 'out-of-stock'}`}>
        {product.inStock ? 'In Stock' : 'Out of Stock'}
      </div>
      <div className="product-meta">
        <span className="category">{product.category}</span>
        <span className="rating">★ {product.rating} ({product.reviews})</span>
      </div>
    </div>
  );
};

export default ProductItem;