const mongoose = require('mongoose');
const { exit } = require('process');
const { Schema, Types } = mongoose;

mongoose.connect('mongodb://127.0.0.1:27017/looplan').then(async () => {
	const users = new Schema({
		_id: Types.ObjectId,
		name: { type: String, minLength: 4, maxLength: 16, },
		password_hash: {
			type: String,
			match: /^\\$argon2i\\$v=19\\$m=\\d{1,10},t=\\d{1,10},p=\\d{1,3}\\$[\\w\\d]{11,84}\\$[\\w\\d+/]{16,86}$/,
			minLength: 50,
			maxLength: 200,
		},
	});
	await mongoose.model('users', users).createCollection();

	const sessions = new Schema({ 
		_id: Types.ObjectId,
		user_id: { type: Types.ObjectId, },
		token: { type: Schema.Types.UUID, }
	});
	await mongoose.model('sessions', sessions).createCollection();

	const schedule = new Schema({
		_id: Types.ObjectId,
		user_id: { type: Types.ObjectId, },
		start_time: { type: Date, },
		end_time: { type: Date, },
		name: { type: String, },
		description: { type: String, },
	});
	await mongoose.model('schedule', schedule).createCollection();
	exit();
})