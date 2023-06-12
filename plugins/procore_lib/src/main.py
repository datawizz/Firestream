
import json

import openai

openai.api_key = "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"





def walk_path():

    path = "/workspace/submodules/procore_lib/src/rest_v1_OAS_all.json"

    with open(path) as f:
        data = json.load(f)

        for key, value in data.items():
            if isinstance(value, dict):

                for key2, value2 in value.items():
                    if isinstance(value2, dict):
                        if value2.get("get"):
                            print(value2)



                            print(value2.get("get").get("responses").get("200").get("content").get("application/json").get("schema"))




def _example():


    project_id = 1

    example = [
        {F"/rest/v1.0/projects/{project_id}/accident_logs": {
      "parameters": [
        {
          "name": "Procore-Company-Id",
          "in": "header",
          "description": "Unique company identifier associated with the Procore User Account.",
          "required": True,
          "schema": {
            "type": "integer"
          }
        },
        {
          "name": "project_id",
          "in": "path",
          "required": True,
          "description": "Unique identifier for the project.",
          "schema": {
            "type": "integer"
          }
        }
      ],
      "get": {
        "summary": "List Accident Logs",
        "description": "Returns all Accident Logs for the current date.\n\nSee [Working with Daily Logs](https://developers.procore.com/documentation/daily-logs) for information on filtering the response using the log\\_date, start\\_date, and end\\_date parameters. Note that if none of the date parameters are provided in the call, only logs from the current date are returned.",
        "tags": [
          "Project Management/Daily Log/Accident Logs"
        ],
        "parameters": [
          {
            "name": "per_page",
            "in": "query",
            "description": "Elements per page",
            "schema": {
              "type": "integer"
            }
          },
          {
            "name": "page",
            "in": "query",
            "description": "Page",
            "schema": {
              "type": "integer"
            }
          },
          {
            "name": "log_date",
            "in": "query",
            "description": "Date of specific logs desired in YYYY-MM-DD format",
            "schema": {
              "type": "string",
              "format": "date"
            }
          },
          {
            "name": "start_date",
            "in": "query",
            "description": "Start date of specific logs desired in YYYY-MM-DD format (use together with end_date)",
            "schema": {
              "type": "string",
              "format": "date"
            }
          },
          {
            "name": "end_date",
            "in": "query",
            "description": "End date of specific logs desired in YYYY-MM-DD format (use together with start_date)",
            "schema": {
              "type": "string",
              "format": "date"
            }
          },
          {
            "name": "filters[created_by_id]",
            "in": "query",
            "description": "Return item(s) created by the specified User ID",
            "style": "form",
            "explode": False,
            "schema": {
              "type": "integer"
            }
          },
          {
            "name": "filters[location_id]",
            "in": "query",
            "description": "Return item(s) with the specified Location IDs.",
            "style": "form",
            "explode": False,
            "schema": {
              "type": "array",
              "items": {
                "type": "integer"
              }
            }
          }
        ],
        "responses": {
          "200": {
            "description": "OK",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": {
                    "type": "object",
                    "properties": {
                      "id": {
                        "type": "integer",
                        "description": "ID",
                        "example": 333675
                      },
                      "comments": {
                        "type": "string",
                        "description": "Additional information about the accident",
                        "example": "There was an accident on the roof"
                      },
                      "created_at": {
                        "type": "string",
                        "format": "date-time",
                        "description": "Created at",
                        "example": "2012-10-23T21:39:40Z"
                      },
                      "date": {
                        "type": "string",
                        "description": "Date that the accident occurred",
                        "format": "date",
                        "example": "2016-05-19"
                      },
                      "datetime": {
                        "type": "string",
                        "format": "date-time",
                        "description": "Estimated UTC datetime of record",
                        "example": "2016-05-19T12:00:00Z"
                      },
                      "deleted_at": {
                        "type": "string",
                        "nullable": True,
                        "format": "date-time",
                        "description": "Deleted at",
                        "example": "2017-07-29T21:39:40Z"
                      },
                      "involved_name": {
                        "type": "string",
                        "description": "Name of the person involved in the accident",
                        "example": "4"
                      },
                      "permissions": {
                        "type": "object",
                        "description": "TBD",
                        "properties": {
                          "can_update": {
                            "type": "boolean",
                            "description": "Can Update",
                            "example": True
                          },
                          "can_delete": {
                            "type": "boolean",
                            "description": "Can Delete",
                            "example": False
                          }
                        }
                      },
                      "involved_company": {
                        "type": "string",
                        "description": "Name of the Company involved in the accident",
                        "example": "Procore Technologies"
                      },
                      "position": {
                        "type": "integer",
                        "description": "Order in which this entry was recorded for the day",
                        "example": 142143
                      },
                      "time_hour": {
                        "type": "integer",
                        "description": "Time of accident - hour",
                        "example": 10,
                        "maximum": 23,
                        "minimum": 0
                      },
                      "time_minute": {
                        "type": "integer",
                        "description": "Time of accident - minute",
                        "example": 15,
                        "maximum": 59,
                        "minimum": 0
                      },
                      "updated_at": {
                        "type": "string",
                        "format": "date-time",
                        "description": "Updated at",
                        "example": "2012-10-24T21:39:40Z"
                      },
                      "created_by": {
                        "type": "object",
                        "properties": {
                          "id": {
                            "type": "integer",
                            "description": "ID",
                            "example": 160586
                          },
                          "login": {
                            "type": "string",
                            "description": "Email",
                            "example": "carl.contractor@example.com"
                          },
                          "name": {
                            "type": "string",
                            "description": "Name",
                            "example": "Carl Contractor"
                          }
                        }
                      },
                      "attachments": {
                        "type": "array",
                        "description": ":filename to be deprecated, use :name",
                        "items": {
                          "type": "object",
                          "properties": {
                            "id": {
                              "type": "integer"
                            },
                            "content_type": {
                              "type": "string",
                              "nullable": True,
                              "example": "image/jpeg"
                            },
                            "name": {
                              "type": "string",
                              "nullable": True,
                              "description": "Use :name, :filename to be deprecated"
                            },
                            "url": {
                              "type": "string"
                            },
                            "filename": {
                              "type": "string",
                              "nullable": True,
                              "description": ":filename to be deprecated, use :name"
                            },
                            "share_url": {
                              "type": "string",
                              "example": "https://example.com/utilities/prostore_local/62d97223fd22a7806c3c7b12fc5ba9ee900d?company_id=15"
                            },
                            "viewable_type": {
                              "type": "string",
                              "example": "image"
                            },
                            "viewable_url": {
                              "type": "string",
                              "example": "https://example.com/15/project/daily_log/viewable_document_image_show?holder_class=AccidentLog\\u0026holder_id=13\\u0026prostore_file_id=76094"
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          },
          "403": {
            "description": "User does not have right permissions",
            "content": {
              "application/json": {
                "schema": {
                  "type": "object",
                  "properties": {
                    "code": {
                      "type": "integer",
                      "format": "int32"
                    },
                    "message": {
                      "type": "string"
                    },
                    "fields": {
                      "type": "string"
                    },
                    "reason": {
                      "description": "A human-readable code providing additional detail on the cause of the error.",
                      "type": "string"
                    },
                    "error": {
                      "description": "Description of the error thrown.",
                      "type": "string",
                      "example": "Invalid Token",
                      "nullable": True
                    }
                  }
                }
              }
            }
          }
        },
        "operationId": "RestV10ProjectsProjectIdAccidentLogsGet"
      },
      "post": {
        "summary": "Create Accident Log",
        "description": "Creates single Accident Log.\n\n#### See - [Daily Log guide](https://developers.procore.com/documentation/daily-logs) - for additional info on\n* Attachments",
        "tags": [
          "Project Management/Daily Log/Accident Logs"
        ],
        "requestBody": {
          "content": {
            "application/json": {
              "schema": {
                "type": "object",
                "required": [
                  "accident_log"
                ],
                "properties": {
                  "accident_log": {
                    "type": "object",
                    "required": [
                      "time_hour",
                      "time_minute"
                    ],
                    "properties": {
                      "comments": {
                        "type": "string",
                        "description": "Additional comments about the accident",
                        "example": "Accident Log comments"
                      },
                      "date": {
                        "type": "string",
                        "format": "date",
                        "description": "Date that the accident occured. Format: YYYY-MM-DD",
                        "example": "2016-05-19"
                      },
                      "datetime": {
                        "type": "string",
                        "format": "date-time",
                        "description": "Datetime of record. Mutually exclusive with the date property.",
                        "example": "2016-05-19T12:00:00Z"
                      },
                      "involved_company": {
                        "type": "string",
                        "description": "Name of the Company involved in the accident",
                        "example": "Procore Technologies"
                      },
                      "involved_name": {
                        "type": "string",
                        "description": "Name of the person involved in the accident",
                        "example": "Roger"
                      },
                      "time_hour": {
                        "type": "integer",
                        "description": "Time of accident - hour",
                        "example": 10,
                        "maximum": 23,
                        "minimum": 0
                      },
                      "time_minute": {
                        "type": "integer",
                        "description": "Time of accident - minute",
                        "example": 15,
                        "maximum": 60,
                        "minimum": 0
                      }
                    }
                  }
                }
              }
            },
            "multipart/form-data": {
              "schema": {
                "type": "object",
                "required": [
                  "accident_log"
                ],
                "properties": {
                  "accident_log": {
                    "type": "object",
                    "required": [
                      "time_hour",
                      "time_minute"
                    ],
                    "properties": {
                      "comments": {
                        "type": "string",
                        "description": "Additional comments about the accident",
                        "example": "Accident Log comments"
                      },
                      "date": {
                        "type": "string",
                        "format": "date",
                        "description": "Date that the accident occured. Format: YYYY-MM-DD",
                        "example": "2016-05-19"
                      },
                      "datetime": {
                        "type": "string",
                        "format": "date-time",
                        "description": "Datetime of record. Mutually exclusive with the date property.",
                        "example": "2016-05-19T12:00:00Z"
                      },
                      "involved_company": {
                        "type": "string",
                        "description": "Name of the Company involved in the accident",
                        "example": "Procore Technologies"
                      },
                      "involved_name": {
                        "type": "string",
                        "description": "Name of the person involved in the accident",
                        "example": "Roger"
                      },
                      "time_hour": {
                        "type": "integer",
                        "description": "Time of accident - hour",
                        "example": 10,
                        "maximum": 23,
                        "minimum": 0
                      },
                      "time_minute": {
                        "type": "integer",
                        "description": "Time of accident - minute",
                        "example": 15,
                        "maximum": 60,
                        "minimum": 0
                      }
                    }
                  },
                  "attachments": {
                    "type": "array",
                    "description": "Accident Log Attachments. To upload attachments you must upload the entire payload as `multipart/form-data` content-type and specify each parameter as form-data together with `attachments[]` as files.",
                    "items": {
                      "type": "string",
                      "format": "binary",
                      "example": "<attachment.png>",
                      "description": "Look at Daily Log Guide for more info."
                    }
                  }
                }
              }
            }
          },
          "required": True
        },
        "responses": {
          "201": {
            "description": "Created",
            "content": {
              "application/json": {
                "schema": {
                  "type": "object",
                  "properties": {
                    "id": {
                      "type": "integer",
                      "description": "ID",
                      "example": 333675
                    },
                    "comments": {
                      "type": "string",
                      "description": "Additional information about the accident",
                      "example": "There was an accident on the roof"
                    },
                    "created_at": {
                      "type": "string",
                      "format": "date-time",
                      "description": "Created at",
                      "example": "2012-10-23T21:39:40Z"
                    },
                    "date": {
                      "type": "string",
                      "description": "Date that the accident occurred",
                      "format": "date",
                      "example": "2016-05-19"
                    },
                    "datetime": {
                      "type": "string",
                      "format": "date-time",
                      "description": "Estimated UTC datetime of record",
                      "example": "2016-05-19T12:00:00Z"
                    },
                    "deleted_at": {
                      "type": "string",
                      "nullable": True,
                      "format": "date-time",
                      "description": "Deleted at",
                      "example": "2017-07-29T21:39:40Z"
                    },
                    "involved_name": {
                      "type": "string",
                      "description": "Name of the person involved in the accident",
                      "example": "4"
                    },
                    "permissions": {
                      "type": "object",
                      "description": "TBD",
                      "properties": {
                        "can_update": {
                          "type": "boolean",
                          "description": "Can Update",
                          "example": True
                        },
                        "can_delete": {
                          "type": "boolean",
                          "description": "Can Delete",
                          "example": False
                        }
                      }
                    },
                    "involved_company": {
                      "type": "string",
                      "description": "Name of the Company involved in the accident",
                      "example": "Procore Technologies"
                    },
                    "position": {
                      "type": "integer",
                      "description": "Order in which this entry was recorded for the day",
                      "example": 142143
                    },
                    "time_hour": {
                      "type": "integer",
                      "description": "Time of accident - hour",
                      "example": 10,
                      "maximum": 23,
                      "minimum": 0
                    },
                    "time_minute": {
                      "type": "integer",
                      "description": "Time of accident - minute",
                      "example": 15,
                      "maximum": 59,
                      "minimum": 0
                    },
                    "updated_at": {
                      "type": "string",
                      "format": "date-time",
                      "description": "Updated at",
                      "example": "2012-10-24T21:39:40Z"
                    },
                    "created_by": {
                      "type": "object",
                      "properties": {
                        "id": {
                          "type": "integer",
                          "description": "ID",
                          "example": 160586
                        },
                        "login": {
                          "type": "string",
                          "description": "Email",
                          "example": "carl.contractor@example.com"
                        },
                        "name": {
                          "type": "string",
                          "description": "Name",
                          "example": "Carl Contractor"
                        }
                      }
                    },
                    "attachments": {
                      "type": "array",
                      "description": ":filename to be deprecated, use :name",
                      "items": {
                        "type": "object",
                        "properties": {
                          "id": {
                            "type": "integer"
                          },
                          "content_type": {
                            "type": "string",
                            "nullable": True,
                            "example": "image/jpeg"
                          },
                          "name": {
                            "type": "string",
                            "nullable": True,
                            "description": "Use :name, :filename to be deprecated"
                          },
                          "url": {
                            "type": "string"
                          },
                          "filename": {
                            "type": "string",
                            "nullable": True,
                            "description": ":filename to be deprecated, use :name"
                          },
                          "share_url": {
                            "type": "string",
                            "example": "https://example.com/utilities/prostore_local/62d97223fd22a7806c3c7b12fc5ba9ee900d?company_id=15"
                          },
                          "viewable_type": {
                            "type": "string",
                            "example": "image"
                          },
                          "viewable_url": {
                            "type": "string",
                            "example": "https://example.com/15/project/daily_log/viewable_document_image_show?holder_class=AccidentLog\\u0026holder_id=13\\u0026prostore_file_id=76094"
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          },
          "400": {
            "description": "Can not create due to errors",
            "content": {
              "application/json": {
                "schema": {
                  "type": "object",
                  "properties": {
                    "code": {
                      "type": "integer",
                      "format": "int32"
                    },
                    "message": {
                      "type": "string"
                    },
                    "fields": {
                      "type": "string"
                    },
                    "reason": {
                      "description": "A human-readable code providing additional detail on the cause of the error.",
                      "type": "string"
                    },
                    "error": {
                      "description": "Description of the error thrown.",
                      "type": "string",
                      "example": "Invalid Token",
                      "nullable": True
                    }
                  }
                }
              }
            }
          },
          "403": {
            "description": "User does not have right permissions",
            "content": {
              "application/json": {
                "schema": {
                  "type": "object",
                  "properties": {
                    "code": {
                      "type": "integer",
                      "format": "int32"
                    },
                    "message": {
                      "type": "string"
                    },
                    "fields": {
                      "type": "string"
                    },
                    "reason": {
                      "description": "A human-readable code providing additional detail on the cause of the error.",
                      "type": "string"
                    },
                    "error": {
                      "description": "Description of the error thrown.",
                      "type": "string",
                      "example": "Invalid Token",
                      "nullable": True
                    }
                  }
                }
              }
            }
          }
        },
        "operationId": "RestV10ProjectsProjectIdAccidentLogsPost"
      }
    }
    }
    ]

    return example







def query_chat(message:str):


    model = "gpt-3.5-turbo"

    system_prompt = """
        You are a developer who creates code in Python. Today you will be creating models using Pydantic.
        I will provide you with a schema in JSON and you will respond with a Python class that matches the schema.
        When you encounter 'format': 'date-time' use the datetime type.
        The JSON data that I pass will include an 'example' field which you should extract into a inline JSON object.
        Please use this example data as a test of the created pydantic class.

        Finally, please only respond with Python code. No not include comments or other text.

        """

    completion = openai.ChatCompletion.create(model=model, messages=[
        {"role": "user", "content": system_prompt},
        {"role": "user", "content": message}]
        )
    print(completion.choices[0].message.content)







if __name__ == "__main__":
    data = """
            "schema": {
                  "type": "array",
                  "items": {
                    "type": "object",
                    "properties": {
                      "id": {
                        "type": "integer",
                        "description": "ID",
                        "example": 333675
                      },
                      "comments": {
                        "type": "string",
                        "description": "Additional information about the accident",
                        "example": "There was an accident on the roof"
                      },
                      "created_at": {
                        "type": "string",
                        "format": "date-time",
                        "description": "Created at",
                        "example": "2012-10-23T21:39:40Z"
                      },
                      "date": {
                        "type": "string",
                        "description": "Date that the accident occurred",
                        "format": "date",
                        "example": "2016-05-19"
                      },
                      "datetime": {
                        "type": "string",
                        "format": "date-time",
                        "description": "Estimated UTC datetime of record",
                        "example": "2016-05-19T12:00:00Z"
                      },
                      "deleted_at": {
                        "type": "string",
                        "nullable": True,
                        "format": "date-time",
                        "description": "Deleted at",
                        "example": "2017-07-29T21:39:40Z"
                      },
                      "involved_name": {
                        "type": "string",
                        "description": "Name of the person involved in the accident",
                        "example": "4"
                      },
                      "permissions": {
                        "type": "object",
                        "description": "TBD",
                        "properties": {
                          "can_update": {
                            "type": "boolean",
                            "description": "Can Update",
                            "example": True
                          },
                          "can_delete": {
                            "type": "boolean",
                            "description": "Can Delete",
                            "example": False
                          }
                        }
                      },
                      "involved_company": {
                        "type": "string",
                        "description": "Name of the Company involved in the accident",
                        "example": "Procore Technologies"
                      },
                      "position": {
                        "type": "integer",
                        "description": "Order in which this entry was recorded for the day",
                        "example": 142143
                      },
                      "time_hour": {
                        "type": "integer",
                        "description": "Time of accident - hour",
                        "example": 10,
                        "maximum": 23,
                        "minimum": 0
                      },
                      "time_minute": {
                        "type": "integer",
                        "description": "Time of accident - minute",
                        "example": 15,
                        "maximum": 59,
                        "minimum": 0
                      },
                      "updated_at": {
                        "type": "string",
                        "format": "date-time",
                        "description": "Updated at",
                        "example": "2012-10-24T21:39:40Z"
                      },
                      "created_by": {
                        "type": "object",
                        "properties": {
                          "id": {
                            "type": "integer",
                            "description": "ID",
                            "example": 160586
                          },
                          "login": {
                            "type": "string",
                            "description": "Email",
                            "example": "carl.contractor@example.com"
                          },
                          "name": {
                            "type": "string",
                            "description": "Name",
                            "example": "Carl Contractor"
                          }
                        }
                      },
                      "attachments": {
                        "type": "array",
                        "description": ":filename to be deprecated, use :name",
                        "items": {
                          "type": "object",
                          "properties": {
                            "id": {
                              "type": "integer"
                            },
                            "content_type": {
                              "type": "string",
                              "nullable": True,
                              "example": "image/jpeg"
                            },
                            "name": {
                              "type": "string",
                              "nullable": True,
                              "description": "Use :name, :filename to be deprecated"
                            },
                            "url": {
                              "type": "string"
                            },
                            "filename": {
                              "type": "string",
                              "nullable": True,
                              "description": ":filename to be deprecated, use :name"
                            },
                            "share_url": {
                              "type": "string",
                              "example": "https://example.com/utilities/prostore_local/62d97223fd22a7806c3c7b12fc5ba9ee900d?company_id=15"
                            },
                            "viewable_type": {
                              "type": "string",
                              "example": "image"
                            },
                            "viewable_url": {
                              "type": "string",
                              "example": "https://example.com/15/project/daily_log/viewable_document_image_show?holder_class=AccidentLog\\u0026holder_id=13\\u0026prostore_file_id=76094"
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
    
    """


    # query_chat(data)
    walk_path()