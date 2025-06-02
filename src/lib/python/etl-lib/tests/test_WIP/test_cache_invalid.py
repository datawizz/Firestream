


import inspect
import re
import hashlib

class MyClass:
    def __init__(self, arg1, arg2):
        self.arg1 = arg1
        self.arg2 = arg2
    
    def my_func(self, x):
        return self.arg1 + self.arg2 + x
    
    def __hash__(self):
        """
        Implementation of a custom hash function for a class.
        This hash function determines changes if any functional part of the class changes.
        Non-functional changes such as comments are ignored.
        """



        # Get the name of the class
        class_name = type(self).__name__

        # Get the names of all base classes
        base_class_names = [base.__name__ for base in type(self).__bases__]

        # Get the source code of all functions in the class, removing comments
        func_source_code = ""
        for name, func in inspect.getmembers(self, inspect.ismethod):
            if func.__qualname__.split(".")[0] == class_name:
                source_lines = inspect.getsourcelines(func)[0]
                source_lines_no_comments = [
                    re.sub(r"(\"\"\"(?:[^\"\\]*(?:\\.[^\"\\]*)*)\"\"\")|('[^'\\]*(?:\\.[^'\\]*)*')|#.*$", "", line)
                    for line in source_lines
                ]
                func_source_code += "".join(source_lines_no_comments)

        # Hash the class name, base class names, and function source code
        hash_input = class_name.encode() + "".join(base_class_names).encode() + func_source_code.encode()
        return int(hashlib.md5(hash_input).hexdigest(), 16)




# example source code
source_code = """

# This is a single-line comment

print("Hello, World!")  # This is a single line comment at the end of valid line of code

'''This is a single line comment using single ticks'''

''' 
This is a multi-line comment
using single ticks
'''

"""

def test_reg():

    # regular expression pattern to match comments in Python source code
    pattern = r'(?<!\w)#[^\'\"\n]*(?:(?:(?<!\\)[\'\"])[^\'\"\n]*(?<!\\)[\'\"]#[^\'\"\n]*)*(?:\n|$)'

    # remove all comments from the source code
    result = re.sub(pattern, '', source_code, flags=re.DOTALL)

    # print the original source code and the result
    print("Original source code:\n" + source_code)
    print("Result after removing comments:\n" + result)

def test_ast():
    import ast

    # parse the source code into an abstract syntax tree
    tree = ast.parse(source_code)

    # define a function to remove comments from the tree
    def remove_comments(node):
        if isinstance(node, ast.Expr) and isinstance(node.value, ast.Constant) and isinstance(node.value.value, str):
            # remove comments that appear at the end of the line
            node.value.value = node.value.value.split("#")[0].strip()
        elif isinstance(node, ast.Expr) and isinstance(node.value, ast.Constant) and isinstance(node.value.value, bytes):
            # handle bytes literals
            node.value.value = node.value.value.split(b"#")[0].strip()
        elif isinstance(node, ast.Expr) and isinstance(node.value, ast.Str):
            # handle multi-line strings with embedded comments
            node.value.s = '\n'.join(line.split('#', 1)[0] for line in node.value.s.split('\n'))
        elif isinstance(node, ast.FunctionDef) or isinstance(node, ast.AsyncFunctionDef):
            # remove comments from function docstrings
            if node.body and isinstance(node.body[0], ast.Expr) and isinstance(node.body[0].value, ast.Constant) and isinstance(node.body[0].value.value, str):
                node.body[0].value.value = node.body[0].value.value.split('#', 1)[0].strip()

        # recursively process all child nodes
        for child_node in ast.iter_child_nodes(node):
            remove_comments(child_node)

    # remove comments from the tree
    remove_comments(tree)

    # convert the modified tree back into Python source code
    new_source_code = ast.unparse(tree)

    # print the original source code and the result
    print("Original source code:\n" + source_code)
    print("Result after removing comments:\n" + new_source_code)



if __name__ == "__main__":

    # # Create an instance of MyClass
    # my_obj = MyClass(1, 2)

    # # Print the hash of the object
    # print(hash(my_obj))

    # # Print the hash of the object again
    # print(hash(my_obj))

    # # Change the value of arg1
    # my_obj.arg1 = 3

    # # Print the hash of the object again
    # print(hash(my_obj))

    # test_reg()
    test_ast()
