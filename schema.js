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
			},
		},
	},
})

db.createCollection("sessions", {
	validator: {
		$jsonSchema: {
			bsonType: "object",
			additionalProperties: false,
			title: "Session Object Validation",
			required: ["user_id", "token"],
			properties: {
				_id: {
					bsonType: "objectId",
				},
				user_id: {
					bsonType: "objectId",
					description: "user associated",
				},
				token: {
					bsonType: "string",
					description: "for bearer token",
				},
			},
		},
	},
});

