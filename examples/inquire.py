from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, Set, Type
from enum import Enum
from InquirerPy import prompt
from InquirerPy.base.control import Choice
import grpc
from google.protobuf.descriptor import EnumDescriptor, FieldDescriptor
from google.protobuf.message import Message

# Import generated proto code
from firestream.v1 import firestream_pb2 as config_pb
from firestream.v1.flow import question_flow_pb2 as flow_pb

@dataclass
class QuestionNode:
    """A node in the question graph."""
    id: str
    field_path: str
    message: str
    question_type: flow_pb.QuestionType
    dependencies: List['QuestionDependency'] = field(default_factory=list)
    validation: Optional['QuestionValidation'] = None
    ui_hints: Optional['QuestionUIHints'] = None
    children: List['QuestionNode'] = field(default_factory=list)
    visited: bool = False

@dataclass
class QuestionDependency:
    """Represents a dependency between questions."""
    field_path: str
    allowed_values: List[str]
    dependency_type: flow_pb.Dependency.DependencyType

@dataclass
class QuestionValidation:
    """Validation rules for a question."""
    required_fields: List[str]
    regex_pattern: Optional[str]
    min_value: Optional[int]
    max_value: Optional[int]
    allowed_values: List[str]
    custom_validator: Optional[str]

@dataclass
class QuestionUIHints:
    """UI hints for question presentation."""
    default_value: Optional[str]
    placeholder: Optional[str]
    is_password: bool
    help_text: Optional[str]
    choices: List[str]

class DynamicQuestionFlow:
    """Manages dynamic question flow based on proto definitions."""

    def __init__(self):
        self.config = {}
        self.visited_fields: Set[str] = set()
        self.question_graph: Optional[QuestionNode] = None

    def build_graph(self, flow_config: flow_pb.QuestionFlow) -> None:
        """Builds question graph from proto config."""
        # Create nodes for all questions
        nodes_by_id = {}
        root_nodes = []

        # First pass: Create all nodes
        for question in flow_config.questions:
            node = self._create_node_from_proto(question)
            nodes_by_id[question.id] = node

            # If no dependencies, it's a root node
            if not question.dependencies:
                root_nodes.append(node)

        # Second pass: Build relationships
        for question in flow_config.questions:
            node = nodes_by_id[question.id]

            # Find parent nodes based on dependencies
            for dep in question.dependencies:
                parent_field = dep.field_path.split('.')[0]  # Get top-level field
                for potential_parent in nodes_by_id.values():
                    if potential_parent.field_path.startswith(parent_field):
                        potential_parent.children.append(node)

        # Store root nodes
        self.root_nodes = root_nodes

    def _create_node_from_proto(self, question: flow_pb.Question) -> QuestionNode:
        """Creates a QuestionNode from a proto Question."""
        return QuestionNode(
            id=question.id,
            field_path=question.field_path,
            message=question.message,
            question_type=question.type,
            dependencies=[
                QuestionDependency(
                    field_path=dep.field_path,
                    allowed_values=list(dep.allowed_values),
                    dependency_type=dep.type
                )
                for dep in question.dependencies
            ],
            validation=QuestionValidation(
                required_fields=list(question.validation.required_fields),
                regex_pattern=question.validation.regex_pattern,
                min_value=question.validation.min_value,
                max_value=question.validation.max_value,
                allowed_values=list(question.validation.allowed_values),
                custom_validator=question.validation.custom_validator
            ) if question.HasField('validation') else None,
            ui_hints=QuestionUIHints(
                default_value=question.ui_hints.default_value,
                placeholder=question.ui_hints.placeholder,
                is_password=question.ui_hints.is_password,
                help_text=question.ui_hints.help_text,
                choices=list(question.ui_hints.choices)
            ) if question.HasField('ui_hints') else None
        )

    def _get_enum_type(self, field_path: str) -> Optional[Type[Enum]]:
        """Gets the corresponding enum type for a field path."""
        enum_mapping = {
            "infrastructure.provider": config_pb.Provider,
            "infrastructure.ingress_type": config_pb.IngressType,
            "infrastructure.security_level": config_pb.SecurityLevel,
            "gcp.region": config_pb.Region,
            "gcp.region_zone": config_pb.RegionZone,
            "gcp.machine_type": config_pb.MachineType,
            "gcp.environment": config_pb.Environment,
            "gcp.storage_class": config_pb.StorageClass,
            "gcp.backup_strategy": config_pb.BackupStrategy,
        }
        return enum_mapping.get(field_path)

    def _get_choices_for_enum(self, enum_type: Type[Enum]) -> List[Choice]:
        """Converts enum type to InquirerPy choices."""
        return [
            Choice(
                value=value,
                name=name.split('_', 1)[1].replace('_', ' ').title()
            )
            for name, value in enum_type.items()
            if not name.endswith('UNSPECIFIED')
        ]

    def _evaluate_dependency(self, dep: QuestionDependency, config: Dict) -> bool:
        """Evaluates if a dependency is satisfied."""
        current_value = self._get_value_from_path(config, dep.field_path)

        if dep.dependency_type == flow_pb.Dependency.DEPENDENCY_TYPE_EQUALS:
            return str(current_value) in dep.allowed_values
        elif dep.dependency_type == flow_pb.Dependency.DEPENDENCY_TYPE_NOT_EQUALS:
            return str(current_value) not in dep.allowed_values
        elif dep.dependency_type == flow_pb.Dependency.DEPENDENCY_TYPE_IN:
            return str(current_value) in dep.allowed_values
        elif dep.dependency_type == flow_pb.Dependency.DEPENDENCY_TYPE_NOT_IN:
            return str(current_value) not in dep.allowed_values
        elif dep.dependency_type == flow_pb.Dependency.DEPENDENCY_TYPE_GREATER_THAN:
            return current_value > float(dep.allowed_values[0])
        elif dep.dependency_type == flow_pb.Dependency.DEPENDENCY_TYPE_LESS_THAN:
            return current_value < float(dep.allowed_values[0])
        return False

    def _should_ask_question(self, node: QuestionNode) -> bool:
        """Determines if a question should be asked based on its dependencies."""
        if not node.dependencies:
            return True

        return all(
            self._evaluate_dependency(dep, self.config)
            for dep in node.dependencies
        )

    def _create_question(self, node: QuestionNode) -> Dict:
        """Creates an InquirerPy question from a node."""
        question_type = "list"
        choices = None

        if node.question_type == flow_pb.QUESTION_TYPE_BOOL:
            question_type = "confirm"
        elif node.question_type == flow_pb.QUESTION_TYPE_ENUM:
            enum_type = self._get_enum_type(node.field_path)
            if enum_type:
                choices = self._get_choices_for_enum(enum_type)

        question = {
            "type": question_type,
            "message": node.message,
            "name": node.field_path,
        }

        if choices:
            question["choices"] = choices

        if node.ui_hints:
            if node.ui_hints.default_value:
                question["default"] = node.ui_hints.default_value
            if node.ui_hints.is_password:
                question["type"] = "password"
            if node.ui_hints.help_text:
                question["help_message"] = node.ui_hints.help_text

        return question

    def _process_node(self, node: QuestionNode) -> None:
        """Processes a single node in the question graph."""
        if node.visited or not self._should_ask_question(node):
            return

        question = self._create_question(node)
        answer = prompt([question])
        self._set_value_from_path(self.config, node.field_path, answer[node.field_path])

        node.visited = True
        self.visited_fields.add(node.field_path)

        # Process child nodes
        for child in node.children:
            self._process_node(child)

    def _get_value_from_path(self, data: Dict, path: str) -> Any:
        """Gets a value from a nested dictionary using a dot-notation path."""
        current = data
        for key in path.split('.'):
            current = current.get(key, {})
        return current

    def _set_value_from_path(self, data: Dict, path: str, value: Any) -> None:
        """Sets a value in a nested dictionary using a dot-notation path."""
        keys = path.split('.')
        current = data
        for key in keys[:-1]:
            current = current.setdefault(key, {})
        current[keys[-1]] = value

    def run_flow(self) -> Dict:
        """Runs the question flow and returns the configuration."""
        for root_node in self.root_nodes:
            self._process_node(root_node)
        return self.config

def main():
    # Example usage
    channel = grpc.insecure_channel('localhost:50051')
    stub = flow_pb.QuestionFlowServiceStub(channel)

    try:
        # Get flow configuration from service
        flow_config = stub.GetQuestionFlow(flow_pb.GetQuestionFlowRequest())

        # Create and run flow
        flow = DynamicQuestionFlow()
        flow.build_graph(flow_config)
        config = flow.run_flow()

        print("\nFinal configuration:")
        print(config)

    except grpc.RpcError as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
