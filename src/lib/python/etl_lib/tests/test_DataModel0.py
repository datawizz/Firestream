

from etl_lib import DataModel
from typing import List, Optional, Dict

class UserInfo(DataModel):
    hobby: List[str]
    last_seen: Optional[int]
    pet_ages: Dict[str, int]



class Orders(DataModel):
    order_id: str
    customer_id: UserInfo




if __name__ == "__main__":
    instance = Orders(order_id="123", customer_id=UserInfo(hobby=["hiking", "swimming"], last_seen=1234567890, pet_ages={"dog": 5, "cat": 3}))
    
    # result = instance.json_schema()
    # print(result)
    result = instance.get_schema(format="AVRO")
    print(result)

    result = instance.get_dependencies()
    print(result)