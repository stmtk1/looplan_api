db.createCollection("users", {
	validator: {
		$jsonSchema: {
			bsonType: "object",
			additionalProperties: false,
			title: "User Object Validation",
			required: ["name", "password_hash"],
			properties: {
				_id: {
					bsonType: "objectId",
				},
				name: {
					bsonType: "string",
					description: "'name must be string and is required'",
					minLength: 4,
					maxLength: 16,
				},
				password_hash: {
					bsonType: "string",
					description: "'password' must be string and is required'",
					minLength: 50,
					maxLength: 200,
					pattern: '^\\$argon2i\\$v=19\\$m=\\d{1,10},t=\\d{1,10},p=\\d{1,3}\\$[\\w\\d]{11,84}\\$[\\w\\d+/]{16,86}$',
				},
				sessions: {
					bsonType: "array",
					description: "for authentication",
					items: {
						additionalProperties: false,
						required: ["token", "user_id", "expired"],
						properties: {
							token: {
								bsonType: "string",
							},
							user_id:  {
								bsonType: "string"
							},
							expired: {
								bsonType: "timestamp",
							}
						},
					},
					additionalItems: false,
				}
			},
		},
	},
})

