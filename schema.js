const mongoose = require('mongoose');
const { exit } = require('process');
const { Schema, Types } = mongoose;

mongoose.connect('mongodb://127.0.0.1:27017/looplan').then(async () => {
	const users = new Schema({
		_id: Types.ObjectId,
		name: { type: String, minLength: 4, maxLength: 16, required: true, },
		password_hash: {
			type: String,
			match: /^\\$argon2i\\$v=19\\$m=\\d{1,10},t=\\d{1,10},p=\\d{1,3}\\$[\\w\\d]{11,84}\\$[\\w\\d+/]{16,86}$/,
			minLength: 50,
			maxLength: 200,
			required: true,
		},
	});
	await mongoose.model('users', users).createCollection();

	const sessions = new Schema({ 
		_id: Types.ObjectId,
		user_id: { type: Types.ObjectId, ref: "users", required: true },
		token: { type: Schema.Types.UUID, required: true }
	});
	await mongoose.model('sessions', sessions).createCollection();

	const schedule_color = new Schema({
		_id: Types.ObjectId,
		color: { type: String, match: /^#[\da-f]{6}$/, required: true },
		name: { type: String, maxLength: 100, required: true, },
	})
	await mongoose.model('schedule_colors', schedule_color).createCollection();

	const schedule = new Schema({
		_id: Types.ObjectId,
		user_id: { type: Types.ObjectId, ref: "users", required: true, },
		color_id: { type: Types.ObjectId, ref: "schedule_colors", required: true, },
		start_time: { type: Date, required: true, },
		end_time: { type: Date, required: true, },
		name: { type: String, minLength: 3, maxLength: 100, required: true, },
		description: { type: String, minLength: 0, maxLength: 1000, required: true, },
	});
	await mongoose.model('schedules', schedule).createCollection();

	exit();
});